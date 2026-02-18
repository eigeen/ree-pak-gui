use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use indexmap::IndexMap;
use parking_lot::Mutex;
use ree_pak_core::{
    filename::FileNameTable,
    pak::PakMetadata,
    pakfile::PakFile,
    utf16_hash::Utf16HashExt,
    write::{FileOptions, PakWriter},
};
use walkdir::WalkDir;

use crate::{
    channel::{PackProgressChannel, PackedFile, PackedFileTree, PackedPak, UnpackProgressChannel},
    common::JsSafeHash,
    error::{Error, Result},
    pak::{
        ExtractOptions, Pak, PakId, PakInfo,
        group::PakGroup,
        tree::{FileTree, RenderTreeNode, RenderTreeOptions},
    },
};

pub type PakHeaderInfo = PakMetadata;

static PAK_SERVICE: OnceLock<PakService> = OnceLock::new();

/// Builder for creating PackedFileTree from packed files
struct PakTreeBuilder {
    paks: IndexMap<String, Vec<PackedFile>>,
}

impl PakTreeBuilder {
    fn new() -> Self {
        Self {
            paks: IndexMap::new(),
        }
    }

    fn add_file(&mut self, pak_path: &str, file_path: Option<String>, hash: u64, size: u64) {
        let pak_files = self.paks.entry(pak_path.to_string()).or_default();
        pak_files.push(PackedFile::new(
            file_path.unwrap_or_else(|| format!("{:16X}", hash)),
            JsSafeHash::from_u64(hash),
            size,
        ));
    }

    fn build(self) -> PackedFileTree {
        let paks = self
            .paks
            .into_iter()
            .map(|(path, files)| PackedPak::new(path, files))
            .collect();

        PackedFileTree::new(paks)
    }
}

pub struct PakService {
    pak_group: Arc<Mutex<PakGroup>>,
    work_thread: Mutex<Option<JoinHandle<()>>>,
    should_terminate: Arc<AtomicBool>,
}

impl PakService {
    pub fn initialize(pak_group: PakGroup) -> &'static Self {
        PAK_SERVICE.get_or_init(|| Self::new(pak_group))
    }

    pub fn get() -> &'static Self {
        PAK_SERVICE.get().unwrap()
    }

    pub fn open_pak(&self, path: &str) -> Result<PakId> {
        let file = File::open(path).map_err(|source| Error::FileIO {
            path: path.to_string(),
            source,
        })?;

        let pakfile = PakFile::from_file(file).map_err(|e| match e {
            ree_pak_core::error::PakError::IO(source) => Error::FileIO {
                path: path.to_string(),
                source,
            },
            other => Error::PakCore(other),
        })?;

        let path_abs = Path::new(path)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(path))
            .display()
            .to_string();
        let pak = Pak::new(&path_abs, pakfile);
        let id = pak.id;

        self.pak_group.lock().add_pak(pak);
        Ok(id)
    }
}

impl PakService {
    fn new(pak_group: PakGroup) -> Self {
        Self {
            pak_group: Arc::new(Mutex::new(pak_group)),
            work_thread: Mutex::new(None),
            should_terminate: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn pak_group(&self) -> Arc<Mutex<PakGroup>> {
        self.pak_group.clone()
    }

    pub fn terminate_work(&self) {
        if let Some(handle) = self.work_thread.lock().take() {
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
            return Err(Error::InvalidOrder(
                "Order list contains unknown pak ids.".to_string(),
            ));
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
    pub fn unpack_optional(
        &self,
        options: &ExtractOptions,
        progress: UnpackProgressChannel,
    ) -> Result<()> {
        if let Some(handle) = &*self.work_thread.lock()
            && !handle.is_finished()
        {
            return Err(Error::UnpackAlreadyRunning);
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
        *self.work_thread.lock() = Some(thread::spawn(move || {
            let mut _pak_group = pak_group.lock();
            let file_name_table = Arc::new(_pak_group.file_name_table().unwrap().clone());
            let output_root = PathBuf::from(&options1.output_path);
            for pak in _pak_group.paks() {
                if should_terminate.load(Ordering::Relaxed) {
                    break;
                }

                let target_hashes = if options1.extract_all {
                    None
                } else {
                    Some(Arc::new(
                        options1
                            .extract_files
                            .iter()
                            .filter_map(|info| {
                                if info.belongs_to == pak.id {
                                    Some(info.hash.hash_u64())
                                } else {
                                    None
                                }
                            })
                            .collect::<std::collections::HashSet<_>>(),
                    ))
                };

                let mut extractor = pak
                    .pakfile
                    .extractor_callback()
                    .file_name_table_arc(file_name_table.clone())
                    .skip_unknown(false)
                    .continue_on_error(true)
                    .cancel_flag(should_terminate.clone());

                if let Some(target_hashes) = target_hashes {
                    extractor =
                        extractor.filter(move |entry, _path| target_hashes.contains(&entry.hash()));
                }

                let progress1 = progress.clone();
                let output_root1 = output_root.clone();
                let result = extractor.run_with_entry_reader(|entry, rel_path, entry_reader| {
                    let rel_path_str = rel_path.to_string_lossy();
                    let out_path = output_root1.join(rel_path);

                    let result = (|| -> std::result::Result<(), ree_pak_core::error::PakError> {
                        let Some(parent) = out_path.parent() else {
                            return Ok(());
                        };
                        if !parent.exists() {
                            std::fs::create_dir_all(parent)?;
                        }

                        // Keep existing behavior: always overwrite.
                        let mut file = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(&out_path)?;

                        std::io::copy(entry_reader, &mut file)?;
                        file.flush()?;

                        if out_path.extension().is_none()
                            && let Some(ext) = entry_reader.determine_extension()
                        {
                            let new_path = out_path.with_extension(ext);
                            std::fs::rename(&out_path, &new_path)?;
                        }

                        Ok(())
                    })();

                    progress1.file_done(
                        rel_path_str.as_ref(),
                        entry.hash(),
                        result.as_ref().err().map(|e| e.to_string()),
                    );

                    if let Err(e) = &result {
                        log::error!("Error processing entry: {}. Path: {}", e, rel_path_str);
                        log::debug!("Entry: {:?}", entry);
                    }

                    result
                });

                if let Err(e) = result {
                    eprintln!("Error unpacking pak: {}", e);
                }
            }
            progress.work_finished();
        }));

        Ok(())
    }

    /// Unpack a specific file from Paks.
    pub fn unpack_file(&self, entry_path: &str, output_path: impl AsRef<Path>) -> Result<()> {
        let mut _pak_group = self.pak_group.lock();
        if _pak_group.paks().is_empty() {
            return Err(Error::NoPaksLoaded);
        }
        if _pak_group.file_name_table().is_none() {
            return Err(Error::MissingFileList);
        }

        // get newest file from paks
        let file_hash = entry_path.hash_mixed();
        for pak in _pak_group.paks().iter().rev() {
            if let Some(entry) = pak
                .pakfile
                .metadata()
                .entries()
                .iter()
                .find(|e| e.hash() == file_hash)
            {
                let mut entry_reader = pak.pakfile.open_entry(entry)?;

                let output_path = output_path.as_ref();
                let file_dir = output_path.parent().unwrap();
                if !file_dir.exists() {
                    std::fs::create_dir_all(file_dir)?;
                }
                let mut file = File::create(output_path)?;
                std::io::copy(&mut entry_reader, &mut file)?;
                return Ok(());
            }
        }

        Err(Error::PakEntryNotFound(entry_path.to_string()))
    }

    pub fn set_file_name_table(&self, table: FileNameTable) {
        self.pak_group.lock().set_file_name_table(table);
    }

    pub fn push_file_paths(&self, paths: Vec<String>) {
        let mut pak_group = self.pak_group.lock();
        let file_list = pak_group.file_name_table_mut();
        if let Some(file_list) = file_list {
            for file_name in paths {
                file_list.push_str(&file_name);
            }
        }
    }

    pub fn pack(
        &self,
        sources: &[impl AsRef<str>],
        output: &str,
        progress: PackProgressChannel,
    ) -> Result<()> {
        if let Some(handle) = &*self.work_thread.lock()
            && !handle.is_finished()
        {
            return Err(Error::PackAlreadyRunning);
        }

        let output_path = PathBuf::from(output);
        if output_path.exists() {
            return Err(Error::FileIO {
                path: output_path.display().to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "Output file already exists",
                ),
            });
        }

        let should_terminate = self.should_terminate.clone();
        let sources = sources
            .iter()
            .map(|s| s.as_ref().to_string())
            .collect::<Vec<_>>();
        let progress1 = progress.clone();
        *self.work_thread.lock() = Some(thread::spawn(move || {
            let main_fn = move || -> Result<PakTreeBuilder> {
                // collect all files and build file tree
                let mut file_manifests: IndexMap<u64, FileManifest> = IndexMap::new();
                let mut tree_builder = PakTreeBuilder::new();

                for source in sources {
                    if should_terminate.load(Ordering::Relaxed) {
                        return Err(Error::Terminated);
                    }

                    let source_path = Path::new(&source);
                    if !source_path.exists() {
                        return Err(Error::FileIO {
                            path: source_path.display().to_string(),
                            source: std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "File not found",
                            ),
                        });
                    }

                    let root_path = source_path.to_path_buf();

                    if source_path.is_file() {
                        // if is file, check its a pak
                        let mut magic = [0; 4];
                        let mut file = File::open(source_path)?;
                        file.read_exact(&mut magic)?;
                        if magic != *b"AKPK" {
                            return Err(Error::FileIO {
                                path: source_path.display().to_string(),
                                source: std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "Not a pak file",
                                ),
                            });
                        }

                        // collect files
                        let header = PakService::get_header(source_path)?;
                        for entry in header.entries() {
                            file_manifests.insert(
                                entry.hash(),
                                FileManifest {
                                    real_path: None,
                                    pak_path: None,
                                    from_pak: Some(source_path.to_path_buf()),
                                },
                            );
                        }
                    } else {
                        WalkDir::new(source_path)
                            .into_iter()
                            .filter(|entry| {
                                entry
                                    .as_ref()
                                    .unwrap()
                                    .metadata()
                                    .is_ok_and(|m| m.is_file())
                            })
                            .try_for_each(|entry| -> Result<()> {
                                let entry = entry.map_err(|e| Error::FileIO {
                                    path: root_path.display().to_string(),
                                    source: std::io::Error::other(e.to_string()),
                                })?;
                                let path = entry.path();
                                let relative_path =
                                    get_relative_path_with_parent(&root_path, path)?;
                                let hash = relative_path.hash_mixed();

                                log::debug!("Adding loose file: {:?}", relative_path);
                                file_manifests.insert(
                                    hash,
                                    FileManifest {
                                        real_path: Some(path.to_path_buf()),
                                        pak_path: Some(relative_path),
                                        from_pak: None,
                                    },
                                );
                                Ok(())
                            })?;
                    }
                }

                // create output pak file
                let output_writer = BufWriter::new(File::create(&output_path)?);
                let mut pak_writer = PakWriter::new(output_writer, file_manifests.len() as u64);
                progress1.work_start(file_manifests.len() as u32);

                // wrapper pak_writer to ensure it is finished
                let mut write_files_into_pak = || -> Result<()> {
                    // write loose files
                    for (&hash, manifest) in &file_manifests {
                        if should_terminate.load(Ordering::Relaxed) {
                            return Err(Error::Terminated);
                        }

                        let Some(real_path) = &manifest.real_path else {
                            continue;
                        };
                        let mut reader = BufReader::new(File::open(real_path)?);
                        let file_size = real_path.metadata()?.len();
                        pak_writer.start_file_hash(hash, FileOptions::default())?;
                        std::io::copy(&mut reader, &mut pak_writer)?;
                        progress1.file_done(real_path.to_str().unwrap());

                        // Add to tree builder
                        tree_builder.add_file(
                            &output_path.to_string_lossy(),
                            manifest.pak_path.clone(),
                            hash,
                            file_size,
                        );
                    }
                    // write pak files. group by from_pak,
                    // and write each pak file
                    let mut pak_files = file_manifests
                        .values()
                        .filter_map(|manifest| manifest.from_pak.clone())
                        .collect::<Vec<_>>();
                    pak_files.sort();
                    pak_files.dedup();
                    for pak_file in pak_files {
                        if should_terminate.load(Ordering::Relaxed) {
                            return Err(Error::Terminated);
                        }

                        let pak = PakFile::from_file(File::open(&pak_file)?)?;
                        for entry in pak.metadata().entries() {
                            if !file_manifests.contains_key(&entry.hash()) {
                                continue;
                            }

                            let mut reader = pak.open_entry(entry)?;
                            pak_writer.start_file_hash(entry.hash(), FileOptions::default())?;
                            std::io::copy(&mut reader, &mut pak_writer)?;
                            progress1.file_done(&format!(
                                "{}:{:16X}",
                                pak_file.display(),
                                entry.hash()
                            ));

                            // Add to tree builder
                            // tree_builder.add_pak_file(
                            //     &output_path.to_string_lossy(),
                            //     &format!("{}:{:16X}", pak_file.display(), entry.hash()),
                            //     entry.hash(),
                            //     entry.uncompressed_size(),
                            // );
                            tree_builder.add_file(
                                &output_path.to_string_lossy(),
                                None,
                                entry.hash(),
                                entry.uncompressed_size(),
                            );
                        }
                    }
                    Ok(())
                };

                let result = write_files_into_pak();
                pak_writer.finish()?;
                result?;

                Ok(tree_builder)
            };

            match main_fn() {
                Ok(tree_builder) => progress.work_finished(tree_builder.build()),
                Err(e) => progress.error(e.to_string()),
            }
        }));

        Ok(())
    }
}

impl PakService {
    pub fn get_header(path: impl AsRef<Path>) -> Result<PakHeaderInfo> {
        let path = path.as_ref();
        // open pak file
        let file = std::fs::File::open(path).map_err(|e| Error::FileIO {
            path: path.to_string_lossy().to_string(),
            source: e,
        })?;
        let mut reader = std::io::BufReader::new(file);
        let metadata = ree_pak_core::read::read_metadata(&mut reader)?;

        Ok(metadata)
    }
}

struct FileManifest {
    real_path: Option<PathBuf>,
    pak_path: Option<String>,
    from_pak: Option<PathBuf>,
}

/// 获取包含父目录名称的相对路径
/// 例如：root_path = "A", file_path = "A/B/C" -> 返回 "A/B/C"
fn get_relative_path_with_parent(root_path: &Path, file_path: &Path) -> Result<String> {
    // ensure inputs are absolute
    if !root_path.is_absolute() || !file_path.is_absolute() {
        return Err(Error::FileIO {
            path: file_path.display().to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "File path is not absolute",
            ),
        });
    }

    // 获取根路径的父目录
    let root_parent = root_path.parent().unwrap_or(Path::new(""));
    let root_parent_str = root_parent.to_string_lossy();
    let file_str = file_path.to_string_lossy();

    let relative_path = if !root_parent_str.is_empty() {
        file_str
            .strip_prefix(root_parent_str.as_ref())
            .unwrap_or_else(|| file_str.as_ref())
    } else {
        file_str.as_ref()
    };

    // Remove leading path separators after strip_prefix
    let relative_path = relative_path
        .trim_start_matches('/')
        .trim_start_matches('\\');

    if Path::new(relative_path).is_absolute() {
        return Err(Error::FileIO {
            path: file_path.display().to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "File path is absolute after strip",
            ),
        });
    }

    Ok(relative_path.replace("\\", "/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_relative_path_with_parent() {
        // 测试包含父目录名称的相对路径
        let root_path = Path::new("C:/Folder/Project");
        let file_path = Path::new("C:/Folder/Project/natives/STM/test.txt");
        let relative_path = get_relative_path_with_parent(root_path, file_path).unwrap();
        assert_eq!(relative_path, "Project/natives/STM/test.txt");

        let root_path = Path::new("C:/Data/MyMod");
        let file_path = Path::new("C:/Data/MyMod/assets/texture.png");
        let relative_path = get_relative_path_with_parent(root_path, file_path).unwrap();
        assert_eq!(relative_path, "MyMod/assets/texture.png");
    }
}
