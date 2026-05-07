use crate::pak::PakId;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Pak error: {0}")]
    PakCore(#[from] ree_pak_core::error::PakError),

    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("IO error: {0}")]
    UpstreamIO(#[from] std::io::Error),
    #[error("File IO error: path = {path}, source = {source}")]
    FileIO {
        path: String,
        source: std::io::Error,
    },
    #[error("Tex error: {0}")]
    Tex(#[from] re_tex::error::Error),
    #[error("Sound BNK error: {0}")]
    SoundBnk(#[from] re_sound::bnk::BnkError),
    #[error("Sound PCK error: {0}")]
    SoundPck(#[from] re_sound::pck::PckError),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

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
    #[error("File tree load already running.")]
    FileTreeAlreadyRunning,
    #[error("Pack progress already running.")]
    PackAlreadyRunning,
    #[error("Texture export already running.")]
    TextureExportAlreadyRunning,
    #[error("Pak writer error: {0}")]
    PackWriter(#[from] ree_pak_core::write::PakWriteError),
    #[error("Terminated.")]
    Terminated,
    #[error("Pak entry not found: {0}")]
    PakEntryNotFound(String),
    #[error("Preview file not supported: {0}")]
    PreviewFileNotSupported(String),
    #[error("Audio file not supported: {0}")]
    AudioFileNotSupported(String),
    #[error("Audio entry not found: {0}")]
    AudioEntryNotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
