use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use nohash::BuildNoHashHasher;
use parking_lot::Mutex;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ree_pak_core::{
    filename::FileNameTable,
    read::{archive::PakArchiveReader, entry::PakEntryReader},
};

use crate::{
    channel::ProgressChannel,
    error::{Error, Result},
    pak::{
        ExtractOptions, Pak, PakId, PakInfo,
        group::PakGroup,
        tree::{FileTree, RenderTreeNode, RenderTreeOptions},
    },
};

pub struct PakService<R> {
    pak_group: Arc<Mutex<PakGroup<R>>>,
    unpack_thread: Mutex<Option<JoinHandle<()>>>,
    should_terminate: Arc<AtomicBool>,
}

impl PakService<BufReader<File>> {
    pub fn open_pak(&self, path: &str) -> Result<PakId> {
        // open pak file
        let file = std::fs::File::open(path).map_err(|e| Error::FileIO {
            path: path.to_string(),
            source: e,
        })?;
        let mut reader = std::io::BufReader::new(file);
        let archive = ree_pak_core::read::read_archive(&mut reader)?;

        // store pak and create id
        let path_abs = std::path::Path::new(path).canonicalize()?;
        let pak = Pak::new(&path_abs.display().to_string(), archive, reader);
        let id: PakId = pak.id;

        self.pak_group.lock().add_pak(pak);
        Ok(id)
    }
}

impl<R> PakService<R>
where
    R: Send + Sync + BufRead + Seek + 'static,
{
    pub fn new(pak_group: PakGroup<R>) -> Self {
        Self {
            pak_group: Arc::new(Mutex::new(pak_group)),
            unpack_thread: Mutex::new(None),
            should_terminate: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn pak_group(&self) -> Arc<Mutex<PakGroup<R>>> {
        self.pak_group.clone()
    }

    pub fn terminate_unpack(&self) {
        if let Some(handle) = self.unpack_thread.lock().take() {
            self.should_terminate.store(true, Ordering::Relaxed);
            let _ = handle.join();
            self.should_terminate.store(false, Ordering::Relaxed);
        }
    }

    pub fn clear_all_paks(&self) {
        self.pak_group.lock().remove_all_paks();
    }

    pub fn list_all_paks(&self) -> Vec<PakInfo> {
        self.pak_group.lock().pak_infos()
    }

    pub fn close_pak(&self, id: PakId) -> Result<()> {
        if self.pak_group.lock().remove_pak(&id).is_none() {
            return Err(Error::PakIdNotFound(id));
        };
        Ok(())
    }

    pub fn order_paks(&self, order: &[PakId]) -> Result<()> {
        let mut pak_group = self.pak_group.lock();
        let paks = pak_group.paks_mut();
        // check if order list is valid
        if order.len() != paks.len() {
            return Err(Error::InvalidOrder(
                "Order list length does not match number of paks.".to_string(),
            ));
        }
        let all_found = order.iter().all(|id| paks.iter().any(|pak| pak.id == *id));
        if !all_found {
            return Err(Error::InvalidOrder("Order list contains unknown pak ids.".to_string()));
        }
        // sort paks by order list
        paks.sort_by_key(|pak| order.iter().position(|id| pak.id == *id).unwrap());
        Ok(())
    }

    pub fn get_pak_info(&self, id: PakId) -> Result<PakInfo> {
        if let Some(pak) = self.pak_group.lock().get_pak(&id) {
            Ok(PakInfo {
                id,
                path: pak.path.to_string(),
            })
        } else {
            Err(Error::PakIdNotFound(id))
        }
    }

    pub fn read_file_tree(&self) -> Result<FileTree> {
        self.pak_group.lock().render_tree_combined()
    }

    pub fn read_file_tree_optimized(&self, options: &RenderTreeOptions) -> Result<RenderTreeNode> {
        let basic_tree = self.pak_group.lock().render_tree_combined()?;
        RenderTreeNode::try_from_file_tree(basic_tree, options)
    }

    /// Unpack all loaded paks with given options.
    ///
    /// Will run in a new thread.
    ///
    /// # Errors
    ///
    /// - No paks or no file list is loaded.
    /// - Unpack already running.
    pub fn unpack_optional(&self, options: &ExtractOptions, progress: ProgressChannel) -> Result<()> {
        if let Some(handle) = &*self.unpack_thread.lock() {
            if !handle.is_finished() {
                return Err(Error::UnpackAlreadyRunning);
            }
        }

        let _pak_group = self.pak_group.lock();
        if _pak_group.paks().is_empty() {
            return Err(Error::NoPaksLoaded);
        }
        if _pak_group.file_name_table().is_none() {
            return Err(Error::MissingFileList);
        }

        let file_count = if options.extract_all {
            _pak_group.total_files() as u32
        } else {
            options.extract_files.len() as u32
        };
        progress.work_start(file_count);

        let should_terminate = self.should_terminate.clone();
        let pak_group = self.pak_group.clone();
        let options1 = options.clone();
        *self.unpack_thread.lock() = Some(thread::spawn(move || {
            let mut _pak_group = pak_group.lock();
            let file_name_table = _pak_group.file_name_table().unwrap().clone();
            let paks = _pak_group.paks_mut();
            for pak in paks {
                if should_terminate.load(Ordering::Relaxed) {
                    break;
                }

                if let Err(e) = unpack_parallel_error_continue(
                    pak,
                    &file_name_table,
                    &options1,
                    progress.clone(),
                    should_terminate.clone(),
                ) {
                    eprintln!("Error unpacking pak: {}", e);
                }
            }
            progress.work_finished();
        }));

        Ok(())
    }

    pub fn set_file_name_table(&self, table: FileNameTable) {
        self.pak_group.lock().set_file_name_table(table);
    }
}

fn unpack_parallel_error_continue<R>(
    pak: &mut Pak<R>,
    file_name_table: &FileNameTable,
    options: &ExtractOptions,
    progress: ProgressChannel,
    should_terminate: Arc<AtomicBool>,
) -> anyhow::Result<()>
where
    R: Read + Seek + Send,
{
    let mut target_files: HashSet<u64, BuildNoHashHasher<u64>> = HashSet::default();
    for info in options.extract_files.iter() {
        if info.belongs_to == pak.id {
            target_files.insert(info.hash.hash_u64());
        }
    }
    if pak.reader.is_none() {
        return Err(anyhow::anyhow!("Pak reader is not set"));
    }

    let archive_reader = Mutex::new(PakArchiveReader::new(pak.reader.take().unwrap(), &pak.archive));

    let output_path = Path::new(&options.output_path);

    // extract files
    let _ = pak
        .archive
        .entries()
        .par_iter()
        .try_for_each(|entry| -> anyhow::Result<()> {
            if should_terminate.load(Ordering::Relaxed) {
                return Err(anyhow::anyhow!("Unpack thread terminated"));
            }
            if !(options.extract_all || target_files.contains(&entry.hash())) {
                return Ok(());
            }

            // get entry reader
            let entry_reader = {
                let mut r = archive_reader.lock();
                (*r).owned_entry_reader(entry.clone())?
            };
            // output file path
            let file_relative_path: PathBuf = file_name_table
                .get_file_name(entry.hash())
                .map(|fname| fname.get_name().to_string())
                .unwrap_or_else(|| format!("_Unknown/{:08X}", entry.hash()))
                .into();
            let output_path = output_path.join(&file_relative_path);

            let result = process_entry(entry_reader, output_path, true);
            if let Err(e) = &result {
                log::error!(
                    "Error processing entry: {}. Path: {:?}",
                    e,
                    file_name_table.get_file_name(entry.hash()).unwrap(),
                );
                log::debug!("Entry: {:?}", entry);
                progress.file_done(file_relative_path.to_str().unwrap(), entry.hash(), Some(e.to_string()));
            } else {
                progress.file_done(file_relative_path.to_str().unwrap(), entry.hash(), None);
            };
            if let Err(e) = result {
                log::error!("Error processing entry: {}", e);
                return Ok(());
            }
            Ok(())
        });

    pak.reader.replace(archive_reader.into_inner().into_inner());

    Ok(())
}

fn process_entry<R>(mut entry_reader: PakEntryReader<R>, output_path: PathBuf, r#override: bool) -> anyhow::Result<()>
where
    R: BufRead + Seek,
{
    let file_dir = output_path.parent().unwrap();

    if !file_dir.exists() {
        std::fs::create_dir_all(file_dir)?;
    }

    let mut data = vec![];
    std::io::copy(&mut entry_reader, &mut data)?;

    let mut file = if r#override {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&output_path)?
    } else {
        OpenOptions::new().create_new(true).write(true).open(&output_path)?
    };
    file.write_all(&data)?;

    // guess unknown file extension
    if output_path.extension().is_none() {
        if let Some(ext) = entry_reader.determine_extension() {
            let new_path = output_path.with_extension(ext);
            std::fs::rename(output_path, new_path)?;
        }
    }

    Ok(())
}
