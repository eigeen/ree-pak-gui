use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use parking_lot::Mutex;
use re_tex::tex::Tex;
use ree_pak_core::utf16_hash::Utf16HashExt;

use crate::{
    TEMP_DIR_NAME,
    error::{Error, Result},
    get_local_dir,
    service::pak::PakService,
};

static PREVIEW_SERVICE: OnceLock<PreviewService> = OnceLock::new();

/// Preview service.
pub struct PreviewService {
    pak_service: &'static PakService<BufReader<File>>,
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
        let file_type = PreviewFileType::from_extension(ext).ok_or(Error::PreviewFileNotSupported(ext.to_string()))?;

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

fn tex_to_png(tex_path: impl AsRef<Path>, png_path: impl AsRef<Path>) -> Result<()> {
    let tex_path = tex_path.as_ref();
    let png_path = png_path.as_ref();

    let mut reader = BufReader::new(File::open(tex_path)?);
    let tex = Tex::from_reader(&mut reader)?;

    let img = tex.to_rgba_image(0)?;
    img.save_with_format(png_path, image::ImageFormat::Png)?;

    Ok(())
}
