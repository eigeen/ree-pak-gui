use std::{
    fs::File,
    io::{BufReader, BufWriter, Read},
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
use serde::Serialize;
use walkdir::WalkDir;

use crate::{
    channel::{
        FileTreeProgressChannel, PackProgressChannel, PackedFile, PackedFileTree, PackedPak,
        UnpackProgressChannel,
    },
    common::JsSafeHash,
    error::{Error, Result},
    pak::{
        ExtractMode, ExtractOptions, Pak, PakId, PakInfo,
        group::PakGroup,
        tree::{FileTree, RenderTreeNode, RenderTreeOptions},
    },
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PakHeaderInfo {
    pub header: PakHeader,
    pub entries: Vec<PakEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PakHeader {
    pub magic: [u8; 4],
    pub major_version: u8,
    pub minor_version: u8,
    pub feature: u32,
    pub total_files: u32,
    pub hash: String,
    pub unk_u32_sig: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PakEntry {
    pub hash_name_lower: u32,
    pub hash_name_upper: u32,
    pub offset: u64,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub compression_type: u8,
    pub encryption_type: String,
    pub checksum: String,
    pub unk_attr: String,
}

impl From<PakMetadata> for PakHeaderInfo {
    fn from(value: PakMetadata) -> Self {
        Self {
            header: PakHeader::from(value.header().clone()),
            entries: value
                .entries()
                .iter()
                .cloned()
                .map(PakEntry::from)
                .collect(),
        }
    }
}

impl From<ree_pak_core::pak::PakHeader> for PakHeader {
    fn from(value: ree_pak_core::pak::PakHeader) -> Self {
        Self {
            magic: value.magic(),
            major_version: value.major_version(),
            minor_version: value.minor_version(),
            feature: u32::from(value.feature().bits()),
            total_files: value.total_files(),
            hash: format!("{:08x}", value.hash()),
            unk_u32_sig: 0,
        }
    }
}

impl From<ree_pak_core::pak::PakEntry> for PakEntry {
    fn from(value: ree_pak_core::pak::PakEntry) -> Self {
        let hash = value.hash();
        Self {
            hash_name_lower: hash as u32,
            hash_name_upper: (hash >> 32) as u32,
            offset: value.offset_raw(),
            compressed_size: value.compressed_size(),
            uncompressed_size: value.uncompressed_size(),
            compression_type: value.compression_type().bits(),
            encryption_type: format!("{:?}", value.encryption_type()),
            checksum: format!("{:016x}", value.checksum()),
            unk_attr: format!("{:016x}", value.all_attr()),
        }
    }
}

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
    file_tree_running: Arc<AtomicBool>,
    unpack_running: Arc<AtomicBool>,
    unpack_should_terminate: Arc<AtomicBool>,
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
            file_tree_running: Arc::new(AtomicBool::new(false)),
            unpack_running: Arc::new(AtomicBool::new(false)),
            unpack_should_terminate: Arc::new(AtomicBool::new(false)),
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

    pub fn terminate_unpack(&self) {
        self.unpack_should_terminate.store(true, Ordering::SeqCst);
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

    pub async fn read_file_tree_optimized_async(
        &self,
        options: RenderTreeOptions,
        progress: FileTreeProgressChannel,
    ) -> Result<()> {
        if self.file_tree_running.swap(true, Ordering::SeqCst) {
            return Err(Error::FileTreeAlreadyRunning);
        }

        progress.work_start();

        let pak_group = self.pak_group.clone();
        let file_tree_running = self.file_tree_running.clone();
        let result = tokio::task::spawn_blocking(move || {
            let basic_tree = pak_group.lock().render_tree_combined()?;
            RenderTreeNode::try_from_file_tree(basic_tree, &options)
        })
        .await;

        file_tree_running.store(false, Ordering::SeqCst);

        match result {
            Ok(Ok(tree)) => {
                progress.work_finished(tree);
                Ok(())
            }
            Ok(Err(error)) => {
                progress.error(error.to_string());
                Err(error)
            }
            Err(error) => {
                let error = Error::Internal(error.to_string());
                progress.error(error.to_string());
                Err(error)
            }
        }
    }

    /// Unpack all loaded paks with given options.
    /// # Errors
    ///
    /// - No paks or no file list is loaded.
    /// - Unpack already running.
    pub async fn unpack_optional(
        &self,
        options: &ExtractOptions,
        progress: UnpackProgressChannel,
    ) -> Result<()> {
        if self.unpack_running.swap(true, Ordering::SeqCst) {
            return Err(Error::UnpackAlreadyRunning);
        }

        {
            let pak_group = self.pak_group.lock();
            if pak_group.paks().is_empty() {
                self.unpack_running.store(false, Ordering::SeqCst);
                return Err(Error::NoPaksLoaded);
            }
            if pak_group.file_name_table().is_none() {
                self.unpack_running.store(false, Ordering::SeqCst);
                return Err(Error::MissingFileList);
            }
        }

        self.unpack_should_terminate.store(false, Ordering::SeqCst);

        let pak_group = self.pak_group.clone();
        let options = options.clone();
        let should_terminate = self.unpack_should_terminate.clone();
        let unpack_running = self.unpack_running.clone();

        let result = tokio::task::spawn_blocking(move || {
            unpack_optional_blocking(pak_group, options, should_terminate, progress)
        })
        .await
        .map_err(|error| Error::Internal(error.to_string()))?;

        unpack_running.store(false, Ordering::SeqCst);
        result
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
                        let header = PakService::get_header_raw(source_path)?;
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

fn unpack_optional_blocking(
    pak_group: Arc<Mutex<PakGroup>>,
    options: ExtractOptions,
    should_terminate: Arc<AtomicBool>,
    progress: UnpackProgressChannel,
) -> Result<()> {
    let file_count = {
        let pak_group = pak_group.lock();
        if options.extract_all {
            pak_group.total_files() as u32
        } else {
            options.extract_files.len() as u32
        }
    };
    progress.work_start(file_count);

    let mut terminated = false;
    let pak_group = pak_group.lock();
    let file_name_table = Arc::new(
        pak_group
            .file_name_table()
            .expect("file name table checked before spawn")
            .clone(),
    );
    let output_root = PathBuf::from(&options.output_path);
    let relative_roots = Arc::new(
        options
            .extract_files
            .iter()
            .map(|info| (info.hash.hash_u64(), info.relative_root.clone()))
            .collect::<std::collections::HashMap<_, _>>(),
    );

    for pak in pak_group.paks() {
        if should_terminate.load(Ordering::Relaxed) {
            terminated = true;
            break;
        }

        let target_hashes = if options.extract_all {
            None
        } else {
            Some(Arc::new(
                options
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
            extractor = extractor.filter(move |entry, _path| target_hashes.contains(&entry.hash()));
        }

        let progress1 = progress.clone();
        let result = extractor
            .on_event(move |event| {
                if let ree_pak_core::extract::ExtractEvent::FileDone { hash, path, error } = event {
                    progress1.file_done(path.to_string_lossy().as_ref(), hash, error);
                }
            })
            .run_with_reader({
                let output_root = output_root.clone();
                let extract_mode = options.mode;
                let relative_roots = relative_roots.clone();
                move |entry, rel_path, reader| {
                    let relative_root = relative_roots
                        .get(&entry.hash())
                        .map(|value| value.as_deref())
                        .flatten();
                    let output_path = match extract_mode {
                        ExtractMode::RelativePath => {
                            output_root.join(build_extract_relative_path(rel_path, relative_root))
                        }
                        ExtractMode::AbsolutePath => output_root.join(rel_path),
                    };

                    if let Some(parent) = output_path.parent()
                        && !parent.exists()
                    {
                        std::fs::create_dir_all(parent)?;
                    }

                    let mut open_options = std::fs::OpenOptions::new();
                    open_options.write(true).create(true);

                    if options.r#override {
                        open_options.truncate(true);
                    } else {
                        open_options.create_new(true);
                    }

                    let mut file = open_options.open(&output_path)?;
                    std::io::copy(reader, &mut file)?;
                    Ok(())
                }
            });

        if let Err(error) = result {
            if should_terminate.load(Ordering::Relaxed) {
                terminated = true;
                break;
            }
            eprintln!("Error unpacking pak: {}", error);
        }
    }

    if terminated {
        progress.error(Error::Terminated.to_string());
        return Err(Error::Terminated);
    }

    progress.work_finished();
    Ok(())
}

impl PakService {
    pub fn get_header_raw(path: impl AsRef<Path>) -> Result<PakMetadata> {
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

    pub fn get_header(path: impl AsRef<Path>) -> Result<PakHeaderInfo> {
        Self::get_header_raw(path).map(Into::into)
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

fn build_extract_relative_path(rel_path: &Path, relative_root: Option<&str>) -> PathBuf {
    let rel_components = path_components(rel_path);
    let root_components = relative_root
        .filter(|root| !root.trim().is_empty())
        .map(path_string_components)
        .filter(|components| !components.is_empty())
        .unwrap_or_default();

    let stripped_components = if !root_components.is_empty()
        && rel_components
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .starts_with(&root_components)
    {
        &rel_components[root_components.len()..]
    } else {
        &rel_components[..]
    };

    stripped_components
        .iter()
        .fold(PathBuf::new(), |mut path, component| {
            path.push(component);
            path
        })
}

fn path_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            std::path::Component::CurDir => None,
            std::path::Component::ParentDir => Some("..".to_string()),
            std::path::Component::Prefix(_) | std::path::Component::RootDir => None,
        })
        .collect()
}

fn path_string_components(path: &str) -> Vec<&str> {
    path.split(['/', '\\'])
        .filter(|component| !component.is_empty())
        .collect()
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

    #[test]
    fn test_build_extract_relative_path_strips_relative_root() {
        let path = build_extract_relative_path(
            Path::new("natives/STM/stage/test.txt"),
            Some("natives/STM"),
        );
        assert_eq!(path, PathBuf::from("stage/test.txt"));
    }

    #[test]
    fn test_build_extract_relative_path_preserves_path_when_root_does_not_match() {
        let path =
            build_extract_relative_path(Path::new("natives/STM/stage/test.txt"), Some("x/y"));
        assert_eq!(path, PathBuf::from("natives/STM/stage/test.txt"));
    }

    #[test]
    fn test_build_extract_relative_path_supports_windows_style_root() {
        let path = build_extract_relative_path(
            Path::new("natives/STM/stage/test.txt"),
            Some(r"natives\STM"),
        );
        assert_eq!(path, PathBuf::from("stage/test.txt"));
    }
}
