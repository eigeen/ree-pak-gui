use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, BufWriter, Read},
    path::{Path, PathBuf},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use parking_lot::Mutex;
use re_tex::tex::Tex;
use ree_pak_core::utf16_hash::Utf16HashExt;

use crate::pak::{ExtractFileInfo, PakId};

use crate::{
    TEMP_DIR_NAME,
    channel::TextureExportProgressChannel,
    error::{Error, Result},
    get_local_dir,
    path_components::PathComponents,
    service::pak::PakService,
};

static PREVIEW_SERVICE: OnceLock<PreviewService> = OnceLock::new();

/// Preview service.
pub struct PreviewService {
    pak_service: &'static PakService,
    temp_dir: PathBuf,
    preview_files: Mutex<HashMap<u64, PathBuf>>,
    export_running: Arc<AtomicBool>,
    should_terminate: Arc<AtomicBool>,
}

impl PreviewService {
    pub fn initialize() -> Result<&'static Self> {
        let temp_dir = get_local_dir().join(TEMP_DIR_NAME);

        if !temp_dir.exists() {
            std::fs::create_dir_all(&temp_dir)?;
        }

        Ok(PREVIEW_SERVICE.get_or_init(|| Self {
            pak_service: PakService::get(),
            temp_dir,
            preview_files: Mutex::new(HashMap::new()),
            export_running: Arc::new(AtomicBool::new(false)),
            should_terminate: Arc::new(AtomicBool::new(false)),
        }))
    }

    pub fn get() -> &'static Self {
        PREVIEW_SERVICE.get().unwrap()
    }

    /// Get preview file path from Pak.
    ///
    /// If preview not found, create a new one.
    pub async fn get_preview_file(&self, hash: u64) -> Result<PathBuf> {
        // get entry path
        let pak_entry_path = {
            let pak_group = self.pak_service.pak_group();
            let pak_group = pak_group.lock();
            let Some(file_name_table) = pak_group.file_name_table() else {
                return Err(Error::MissingFileList);
            };

            file_name_table
                .get_file_name(hash)
                .map(|p| p.to_string().unwrap())
                .ok_or_else(|| Error::PakEntryNotFound(hash.to_string()))?
        };

        // check file type
        let file_type = preview_file_type_from_path(&pak_entry_path).ok_or_else(|| {
            Error::PreviewFileNotSupported(preview_type_error_hint(&pak_entry_path))
        })?;

        // if preview file exists, return it
        if let Some(path) = self.get_existing_preview_file(&pak_entry_path) {
            return Ok(path);
        }

        // create new preview file
        let pak_service = self.pak_service;
        let pak_entry_path1 = pak_entry_path.to_string();
        let temp_dir = self.temp_dir.clone();

        let path = tokio::task::spawn_blocking(move || -> Result<PathBuf> {
            let entry_path = Path::new(&pak_entry_path1);
            let file_name = entry_path.file_name().unwrap_or_default();
            let raw_output_path = temp_dir.join(format!(
                "{}-{}",
                pak_entry_path1.hash_mixed(),
                file_name.to_string_lossy()
            ));

            // unpack raw file
            pak_service.unpack_file(&pak_entry_path1, &raw_output_path)?;

            // convert to preview format
            let path = match file_type {
                PreviewFileType::Tex => {
                    let png_path = raw_output_path.with_extension("png");
                    tex_to_png(&raw_output_path, &png_path)?;
                    png_path
                }
            };

            Ok(path)
        })
        .await
        .map_err(|e| Error::Internal(e.to_string()))??;

        // store preview file
        let id = pak_entry_path.hash_mixed();
        self.preview_files.lock().insert(id, path.clone());

        Ok(path)
    }

    fn get_existing_preview_file(&self, pak_entry_path: &str) -> Option<PathBuf> {
        let id = pak_entry_path.hash_mixed();

        if let Some(path) = self.preview_files.lock().get(&id) {
            if !path.exists() {
                return None;
            }
            return Some(path.clone());
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreviewFileType {
    Tex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureExportFormat {
    Dds,
    Png,
}

impl PreviewFileType {
    /// Get preview file type from extension.
    ///
    /// Example: tex -> Tex
    fn from_extension(extension: &str) -> Option<Self> {
        match extension {
            "tex" => Some(Self::Tex),
            _ => None,
        }
    }
}

impl TextureExportFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Dds => "dds",
            Self::Png => "png",
        }
    }
}

impl PreviewService {
    pub async fn export_texture_files(
        &self,
        format: TextureExportFormat,
        output_dir: impl AsRef<Path>,
        files: &[ExtractFileInfo],
        progress: TextureExportProgressChannel,
    ) -> Result<usize> {
        if self.export_running.swap(true, Ordering::SeqCst) {
            return Err(Error::TextureExportAlreadyRunning);
        }

        self.should_terminate.store(false, Ordering::SeqCst);

        let output_dir = output_dir.as_ref().to_path_buf();
        let files = files.to_vec();
        let pak_service = self.pak_service;
        let should_terminate = self.should_terminate.clone();
        let export_running = self.export_running.clone();

        let result = tokio::task::spawn_blocking(move || {
            export_texture_files_blocking(
                pak_service,
                format,
                &output_dir,
                &files,
                should_terminate,
                progress,
            )
        })
        .await
        .map_err(|error| Error::Internal(error.to_string()))?;

        export_running.store(false, Ordering::SeqCst);
        Ok(result?)
    }

    pub fn terminate_export(&self) {
        self.should_terminate.store(true, Ordering::SeqCst);
    }
}

fn export_texture_files_blocking(
    pak_service: &PakService,
    format: TextureExportFormat,
    output_dir: &Path,
    files: &[ExtractFileInfo],
    should_terminate: Arc<AtomicBool>,
    progress: TextureExportProgressChannel,
) -> Result<usize> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }

    progress.work_start(files.len() as u32);

    let export_plan = build_texture_export_plan(pak_service, output_dir, files, format)?;
    let mut exported = 0usize;

    for source in &export_plan.sources {
        if should_terminate.load(Ordering::Relaxed) {
            progress.error(Error::Terminated.to_string());
            return Err(Error::Terminated);
        }

        let Some(task_map) = export_plan.tasks_by_pak.get(&source.id) else {
            continue;
        };

        let target_hashes = Arc::new(task_map.keys().copied().collect::<HashSet<_>>());
        let task_map = Arc::new(task_map.clone());
        let progress1 = progress.clone();
        let report = source
            .pakfile
            .extractor_callback()
            .file_name_table_arc(export_plan.file_name_table.clone())
            .skip_unknown(false)
            .continue_on_error(true)
            .cancel_flag(should_terminate.clone())
            .filter({
                let target_hashes = Arc::clone(&target_hashes);
                move |entry, _path| target_hashes.contains(&entry.hash())
            })
            .on_event(move |event| {
                if let ree_pak_core::extract::ExtractEvent::FileDone { path, .. } = event {
                    progress1.file_done(path.to_string_lossy().as_ref());
                }
            })
            .run_with_reader({
                let task_map = Arc::clone(&task_map);
                move |entry, _rel_path, reader| {
                    let Some(task) = task_map.get(&entry.hash()) else {
                        return Err(std::io::Error::other(format!(
                            "missing export task for {:016X}",
                            entry.hash()
                        ))
                        .into());
                    };

                    if let Some(parent) = task.output_path.parent()
                        && !parent.exists()
                    {
                        std::fs::create_dir_all(parent)?;
                    }

                    match task.file_type {
                        PreviewFileType::Tex => {
                            export_tex_reader(reader, &task.output_path, format)?;
                            Ok(())
                        }
                    }
                }
            })?;

        exported += report.extracted;

        for (hash, path, error) in report.errors {
            log::error!(
                "texture export skipped: hash={:016X}, entry_path={}, error={}",
                hash,
                path.display(),
                error
            );
        }

        if should_terminate.load(Ordering::Relaxed) {
            progress.error(Error::Terminated.to_string());
            return Err(Error::Terminated);
        }
    }

    log::info!(
        "texture export finished: requested={}, exported={}, skipped={}",
        export_plan.task_count,
        exported,
        export_plan.task_count.saturating_sub(exported)
    );

    progress.work_finished();

    Ok(exported)
}

#[derive(Debug, Clone)]
struct TextureExportTask {
    output_path: PathBuf,
    file_type: PreviewFileType,
}

struct TextureExportSource {
    id: PakId,
    pakfile: Arc<ree_pak_core::pakfile::PakFile>,
}

struct TextureExportPlan {
    file_name_table: Arc<ree_pak_core::filename::FileNameTable>,
    sources: Vec<TextureExportSource>,
    tasks_by_pak: HashMap<PakId, HashMap<u64, TextureExportTask>>,
    task_count: usize,
}

fn build_texture_export_plan(
    pak_service: &PakService,
    output_dir: &Path,
    files: &[ExtractFileInfo],
    format: TextureExportFormat,
) -> Result<TextureExportPlan> {
    let (file_name_table, sources) = {
        let pak_group = pak_service.pak_group();
        let pak_group = pak_group.lock();
        let Some(file_name_table) = pak_group.file_name_table() else {
            return Err(Error::MissingFileList);
        };

        let sources = pak_group
            .paks()
            .iter()
            .map(|pak| TextureExportSource {
                id: pak.id,
                pakfile: Arc::clone(&pak.pakfile),
            })
            .collect::<Vec<_>>();

        (Arc::new(file_name_table.clone()), sources)
    };

    let mut used_paths = HashSet::new();
    let mut tasks_by_pak = HashMap::<PakId, HashMap<u64, TextureExportTask>>::new();

    for file in files {
        let entry_path = file_name_table
            .get_file_name(file.hash.hash_u64())
            .map(|path| path.to_string().unwrap())
            .ok_or_else(|| Error::PakEntryNotFound(file.hash.hash_u64().to_string()))?;
        let file_type = preview_file_type_from_path(&entry_path)
            .ok_or_else(|| Error::PreviewFileNotSupported(preview_type_error_hint(&entry_path)))?;

        let output_path = output_dir.join(build_texture_output_path(&entry_path, file, format));
        let output_path = ensure_unique_path(output_path, file.hash.hash_u64(), &mut used_paths);

        tasks_by_pak.entry(file.belongs_to).or_default().insert(
            file.hash.hash_u64(),
            TextureExportTask {
                output_path,
                file_type,
            },
        );
    }

    Ok(TextureExportPlan {
        file_name_table,
        sources,
        tasks_by_pak,
        task_count: files.len(),
    })
}

fn build_texture_output_path(
    entry_path: &str,
    file: &ExtractFileInfo,
    format: TextureExportFormat,
) -> PathBuf {
    let relative_path = build_relative_output_path(entry_path, file.relative_root.as_deref());
    let output_file_name = relative_path
        .file_name()
        .map(|name| build_texture_file_name(&name.to_string_lossy(), format, file.hash.hash_u64()))
        .unwrap_or_else(|| format!("{:016X}.{}", file.hash.hash_u64(), format.extension()));

    let mut output_path = relative_path;
    output_path.set_file_name(output_file_name);
    output_path
}

fn tex_to_png(tex_path: impl AsRef<Path>, png_path: impl AsRef<Path>) -> Result<()> {
    let tex_path = tex_path.as_ref();
    let png_path = png_path.as_ref();

    let mut reader = BufReader::new(File::open(tex_path)?);
    let tex = Tex::from_reader(&mut reader)?;

    let img = tex.to_rgba_image(0)?;
    img.save_with_format(png_path, image::ImageFormat::Png)?;

    Ok(())
}

fn export_tex_reader(
    reader: &mut dyn Read,
    output_path: impl AsRef<Path>,
    format: TextureExportFormat,
) -> std::result::Result<(), std::io::Error> {
    let mut buffered = BufReader::new(reader);
    let tex = Tex::from_reader(&mut buffered).map_err(std::io::Error::other)?;
    let output_path = output_path.as_ref();

    match format {
        TextureExportFormat::Dds => {
            let dds = tex
                .to_dds(tex.header.mipmap_count as usize)
                .map_err(std::io::Error::other)?;
            let mut writer = BufWriter::new(File::create(output_path)?);
            dds.write(&mut writer).map_err(std::io::Error::other)?;
        }
        TextureExportFormat::Png => {
            let img = tex.to_rgba_image(0).map_err(std::io::Error::other)?;
            img.save_with_format(output_path, image::ImageFormat::Png)
                .map_err(std::io::Error::other)?;
        }
    }

    Ok(())
}

fn ensure_unique_path(path: PathBuf, hash: u64, used_paths: &mut HashSet<PathBuf>) -> PathBuf {
    if used_paths.insert(path.clone()) {
        return path;
    }

    let stem = path
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("{hash:016X}"));
    let extension = path
        .extension()
        .map(|value| value.to_string_lossy().to_string());
    let unique_name = match extension {
        Some(extension) => format!("{stem}-{hash:016X}.{extension}"),
        None => format!("{stem}-{hash:016X}"),
    };
    let unique_path = path.with_file_name(unique_name);
    used_paths.insert(unique_path.clone());
    unique_path
}

fn build_relative_output_path(entry_path: &str, relative_root: Option<&str>) -> PathBuf {
    let rel_components = path_string_components(entry_path);
    let root_components = relative_root
        .filter(|root| !root.trim().is_empty())
        .map(path_string_components)
        .unwrap_or_default();

    let stripped_components =
        if !root_components.is_empty() && rel_components.starts_with(&root_components) {
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

fn path_string_components(path: &str) -> Vec<&str> {
    path.split(['/', '\\'])
        .filter(|component| !component.is_empty())
        .collect()
}

fn build_texture_file_name(file_name: &str, format: TextureExportFormat, hash: u64) -> String {
    let base = PathComponents::parse(file_name)
        .and_then(|components| {
            Path::new(components.raw_path())
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
        })
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| fallback_texture_file_stem(file_name, hash));

    format!("{base}.{}", format.extension())
}

fn path_file_name(path: &str) -> &str {
    path.rsplit(['/', '\\'])
        .find(|segment| !segment.is_empty())
        .unwrap_or(path)
}

fn preview_file_type_from_path(path: &str) -> Option<PreviewFileType> {
    let components = PathComponents::parse(path)?;
    PreviewFileType::from_extension(components.extension()?)
}

fn fallback_texture_file_stem(file_name: &str, hash: u64) -> String {
    Path::new(file_name)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| format!("{hash:016X}"))
}

fn preview_type_error_hint(path: &str) -> String {
    let file_name = path_file_name(path);
    file_name
        .rsplit('.')
        .next()
        .filter(|segment| !segment.is_empty())
        .unwrap_or(file_name)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_relative_output_path_strips_relative_root() {
        let path = build_relative_output_path(
            "natives/STM/_Develop/Personal/test/foo.tex.241106027",
            Some("natives/STM"),
        );
        assert_eq!(
            path,
            PathBuf::from("_Develop/Personal/test/foo.tex.241106027")
        );
    }

    #[test]
    fn test_build_texture_file_name_replaces_tex_suffix() {
        let name = build_texture_file_name("foo.tex.241106027", TextureExportFormat::Png, 1);
        assert_eq!(name, "foo.png");
    }

    #[test]
    fn test_preview_file_type_supports_numeric_suffix() {
        let file_type = preview_file_type_from_path("foo.tex.241106027");
        assert_eq!(file_type, Some(PreviewFileType::Tex));
    }

    #[test]
    fn test_preview_file_type_supports_version_and_tag_suffix() {
        let file_type = preview_file_type_from_path("foo.tex.241106027.X64");
        assert_eq!(file_type, Some(PreviewFileType::Tex));
    }

    #[test]
    fn test_build_texture_file_name_strips_version_and_tag_suffixes() {
        let name = build_texture_file_name("foo.tex.241106027.X64", TextureExportFormat::Png, 1);
        assert_eq!(name, "foo.png");
    }
}
