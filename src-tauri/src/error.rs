use crate::pak::PakId;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Pak error: {0}")]
    PakCore(#[from] ree_pak_core::error::PakError),

    #[error("IO error: {0}")]
    UpstreamIO(#[from] std::io::Error),
    #[error("File IO error: path = {path}, source = {source}")]
    FileIO { path: String, source: std::io::Error },

    #[error("Missing file list. Please load a file list first.")]
    MissingFileList,

    #[error("Pak ID not found: id = {0:?}")]
    PakIdNotFound(PakId),
    #[error("No Paks loaded.")]
    NoPaksLoaded,
    #[error("Invalid pak order: {0}")]
    InvalidOrder(String),
    #[error("Unpack progress already running.")]
    UnpackAlreadyRunning,

    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}
