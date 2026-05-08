mod container;
mod export;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::{
    TEMP_DIR_NAME,
    channel::AudioExportProgressChannel,
    common::JsSafeHash,
    error::{Error, Result},
    get_local_dir,
    pak::PakId,
    service::pak::PakService,
};

use self::container::{
    ExtractedWem, audio_container_kind_from_path, audio_type_error_hint,
    build_temp_container_file_name, build_temp_wem_file_name, extract_wems_from_file,
    list_container_from_file,
};

static AUDIO_SERVICE: OnceLock<AudioService> = OnceLock::new();

pub struct AudioService {
    pak_service: &'static PakService,
    temp_dir: PathBuf,
    container_files: Mutex<HashMap<u64, PathBuf>>,
    export_running: Arc<AtomicBool>,
    export_should_terminate: Arc<AtomicBool>,
}

impl AudioService {
    pub fn initialize() -> Result<&'static Self> {
        let temp_dir = get_local_dir().join(TEMP_DIR_NAME).join("audio");
        std::fs::create_dir_all(&temp_dir)?;

        Ok(AUDIO_SERVICE.get_or_init(|| Self {
            pak_service: PakService::get(),
            temp_dir,
            container_files: Mutex::new(HashMap::new()),
            export_running: Arc::new(AtomicBool::new(false)),
            export_should_terminate: Arc::new(AtomicBool::new(false)),
        }))
    }

    pub fn get() -> &'static Self {
        AUDIO_SERVICE.get().unwrap()
    }

    pub fn list_container(&self, source: AudioSourceRef) -> Result<AudioContainerInfo> {
        let source_file = self.resolve_source_file(source)?;
        let container_path = self.ensure_container_file(
            source.hash.hash_u64(),
            &source_file.path,
            source_file.kind,
        )?;

        list_container_from_file(source_file.path, source_file.kind, &container_path)
    }

    pub fn extract_wems(&self, options: AudioExtractBatchOptions) -> Result<Vec<PathBuf>> {
        let output_dir = self.resolve_output_dir(options.output_dir);
        self.extract_wems_to_dir(options.source, &options.indices, &output_dir)
    }

    pub fn extract_wavs(&self, options: AudioExtractBatchOptions) -> Result<Vec<PathBuf>> {
        export::extract_wavs(self, options)
    }

    pub fn extract_wavs_with_progress(
        &self,
        options: AudioExtractBatchOptions,
        progress: AudioExportProgressChannel,
    ) -> Result<Vec<PathBuf>> {
        if self.export_running.swap(true, Ordering::SeqCst) {
            return Err(Error::AudioExportAlreadyRunning);
        }

        self.export_should_terminate.store(false, Ordering::SeqCst);
        let progress_for_error = progress.clone();
        let result = export::extract_wavs_with_progress(self, options, progress);
        self.export_running.store(false, Ordering::SeqCst);

        if let Err(error) = &result {
            progress_for_error.error(error.to_string());
        }

        result
    }

    pub fn terminate_extract(&self) {
        self.export_should_terminate.store(true, Ordering::SeqCst);
    }

    pub(super) fn export_should_terminate(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.export_should_terminate)
    }

    pub(super) fn resolve_output_dir(&self, output_dir: Option<String>) -> PathBuf {
        output_dir
            .filter(|path| !path.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.temp_dir.clone())
    }

    pub(super) fn extract_wems_to_dir(
        &self,
        source: AudioSourceRef,
        indices: &[usize],
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let source_hash = source.hash.hash_u64();
        let source_file = self.resolve_source_file(source)?;
        let container_path =
            self.ensure_container_file(source_hash, &source_file.path, source_file.kind)?;
        let wems = extract_wems_from_file(source_file.kind, &container_path, indices)?;

        write_wems_to_dir(source_hash, wems, output_dir)
    }

    fn resolve_source_file(&self, source: AudioSourceRef) -> Result<AudioSourceFile> {
        let path = self
            .pak_service
            .get_entry_path_by_hash(source.hash.hash_u64())?;
        let kind = audio_container_kind_from_path(&path)
            .ok_or_else(|| Error::AudioFileNotSupported(audio_type_error_hint(&path)))?;

        Ok(AudioSourceFile { path, kind })
    }

    fn ensure_container_file(
        &self,
        source_hash: u64,
        source_path: &str,
        kind: AudioContainerKind,
    ) -> Result<PathBuf> {
        let mut container_files = self.container_files.lock();

        if let Some(path) = container_files.get(&source_hash)
            && path.exists()
        {
            return Ok(path.clone());
        }

        std::fs::create_dir_all(&self.temp_dir)?;
        let output_path = self.temp_dir.join(build_temp_container_file_name(
            source_hash,
            source_path,
            kind,
        ));
        self.pak_service.unpack_file(source_path, &output_path)?;
        container_files.insert(source_hash, output_path.clone());

        Ok(output_path)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSourceRef {
    pub hash: JsSafeHash,
    pub belongs_to: PakId,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioExtractBatchOptions {
    pub source: AudioSourceRef,
    pub indices: Vec<usize>,
    pub output_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioContainerInfo {
    pub source_path: String,
    pub container_kind: AudioContainerKind,
    pub entries: Vec<AudioEntryInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioContainerKind {
    Bnk,
    Pck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioEntryInfo {
    pub index: usize,
    pub wem_id: u32,
    pub offset: u64,
    pub size: u64,
    pub language_id: Option<u32>,
}

struct AudioSourceFile {
    path: String,
    kind: AudioContainerKind,
}

fn write_wems_to_dir(
    source_hash: u64,
    wems: Vec<ExtractedWem>,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(output_dir)?;

    wems.into_iter()
        .map(|wem| write_wem_to_dir(source_hash, wem, output_dir))
        .collect()
}

fn write_wem_to_dir(source_hash: u64, wem: ExtractedWem, output_dir: &Path) -> Result<PathBuf> {
    let output_path = output_dir.join(build_temp_wem_file_name(source_hash, wem.index, wem.wem_id));
    std::fs::write(&output_path, wem.data)?;
    Ok(output_path)
}
