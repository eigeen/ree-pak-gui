use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use parking_lot::Mutex;
use re_tex::tex::Tex;
use ree_pak_core::utf16_hash::Utf16HashExt;
use tempfile::NamedTempFile;

use crate::pak::ExtractFileInfo;

use crate::{
    TEMP_DIR_NAME,
    error::{Error, Result},
    get_local_dir,
    service::pak::PakService,
};

static PREVIEW_SERVICE: OnceLock<PreviewService> = OnceLock::new();

/// Preview service.
pub struct PreviewService {
    pak_service: &'static PakService,
    temp_dir: PathBuf,
    preview_files: Mutex<HashMap<u64, PathBuf>>,
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
        // example: path/to/file.tex.241106027 -> tex
        let ext = pak_entry_path.split('.').rev().nth(1).unwrap_or_default();
        let file_type = PreviewFileType::from_extension(ext)
            .ok_or(Error::PreviewFileNotSupported(ext.to_string()))?;

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
    ) -> Result<usize> {
        let output_dir = output_dir.as_ref().to_path_buf();
        let files = files.to_vec();
        let temp_dir = self.temp_dir.clone();
        let pak_service = self.pak_service;

        tokio::task::spawn_blocking(move || {
            export_texture_files_blocking(pak_service, &temp_dir, format, &output_dir, &files)
        })
        .await
        .map_err(|error| Error::Internal(error.to_string()))?
    }
}

fn export_texture_files_blocking(
    pak_service: &PakService,
    temp_dir: &Path,
    format: TextureExportFormat,
    output_dir: &Path,
    files: &[ExtractFileInfo],
) -> Result<usize> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }

    let mut exported = 0usize;
    let mut used_paths = HashSet::new();

    for file in files {
        let entry_path = resolve_entry_path(pak_service, file.hash.hash_u64())?;
        let extension = entry_path.split('.').rev().nth(1).unwrap_or_default();
        let file_type = PreviewFileType::from_extension(extension)
            .ok_or_else(|| Error::PreviewFileNotSupported(extension.to_string()))?;

        let output_path = output_dir.join(build_texture_output_path(&entry_path, file, format));
        let output_path = ensure_unique_path(output_path, file.hash.hash_u64(), &mut used_paths);
        if let Some(parent) = output_path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
        }

        let temp_file = NamedTempFile::new_in(temp_dir)?;
        if let Err(error) = pak_service.unpack_file(&entry_path, temp_file.path()) {
            log::error!(
                "texture export skipped: unpack failed: hash={:016X}, entry_path={}, output_path={}, error={}",
                file.hash.hash_u64(),
                entry_path,
                output_path.display(),
                error
            );
            continue;
        }

        let export_result = match file_type {
            PreviewFileType::Tex => export_tex_file(temp_file.path(), &output_path, format),
        };
        if let Err(error) = export_result {
            log::error!(
                "texture export skipped: convert failed: hash={:016X}, entry_path={}, output_path={}, error={}",
                file.hash.hash_u64(),
                entry_path,
                output_path.display(),
                error
            );
            continue;
        }

        exported += 1;
    }

    log::info!(
        "texture export finished: requested={}, exported={}, skipped={}",
        files.len(),
        exported,
        files.len().saturating_sub(exported)
    );

    Ok(exported)
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

fn tex_to_dds(tex_path: impl AsRef<Path>, dds_path: impl AsRef<Path>) -> Result<()> {
    let tex_path = tex_path.as_ref();
    let dds_path = dds_path.as_ref();

    let mut reader = BufReader::new(File::open(tex_path)?);
    let tex = Tex::from_reader(&mut reader)?;
    let dds = tex.to_dds(tex.header.mipmap_count as usize)?;

    let mut writer = BufWriter::new(File::create(dds_path)?);
    dds.write(&mut writer)
        .map_err(|error| Error::Internal(error.to_string()))?;

    Ok(())
}

fn export_tex_file(
    tex_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    format: TextureExportFormat,
) -> Result<()> {
    match format {
        TextureExportFormat::Dds => tex_to_dds(tex_path, output_path),
        TextureExportFormat::Png => tex_to_png(tex_path, output_path),
    }
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

fn resolve_entry_path(pak_service: &PakService, hash: u64) -> Result<String> {
    let pak_group = pak_service.pak_group();
    let pak_group = pak_group.lock();
    let Some(file_name_table) = pak_group.file_name_table() else {
        return Err(Error::MissingFileList);
    };

    file_name_table
        .get_file_name(hash)
        .map(|p| p.to_string().unwrap())
        .ok_or_else(|| Error::PakEntryNotFound(hash.to_string()))
}

fn path_string_components(path: &str) -> Vec<&str> {
    path.split(['/', '\\'])
        .filter(|component| !component.is_empty())
        .collect()
}

fn build_texture_file_name(file_name: &str, format: TextureExportFormat, hash: u64) -> String {
    let parts = file_name.split('.').collect::<Vec<_>>();
    let base = if parts.len() >= 3
        && parts[parts.len() - 2].eq_ignore_ascii_case("tex")
        && parts[parts.len() - 1].chars().all(|ch| ch.is_ascii_digit())
    {
        parts[..parts.len() - 2].join(".")
    } else if parts.len() >= 2 {
        parts[..parts.len() - 1].join(".")
    } else if !file_name.is_empty() {
        file_name.to_string()
    } else {
        format!("{hash:016X}")
    };

    format!("{base}.{}", format.extension())
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
}
