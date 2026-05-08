use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

use parking_lot::Mutex;
use re_sound::{bnk::BnkRaw, pck::Pck};
use serde::{Deserialize, Serialize};

use crate::{
    TEMP_DIR_NAME,
    common::JsSafeHash,
    error::{Error, Result},
    external_tools, get_local_dir,
    pak::PakId,
    path_components::PathComponents,
    service::pak::PakService,
};

static AUDIO_SERVICE: OnceLock<AudioService> = OnceLock::new();

pub struct AudioService {
    pak_service: &'static PakService,
    temp_dir: PathBuf,
    container_files: Mutex<HashMap<u64, PathBuf>>,
}

impl AudioService {
    pub fn initialize() -> Result<&'static Self> {
        let temp_dir = get_local_dir().join(TEMP_DIR_NAME).join("audio");

        if !temp_dir.exists() {
            std::fs::create_dir_all(&temp_dir)?;
        }

        Ok(AUDIO_SERVICE.get_or_init(|| Self {
            pak_service: PakService::get(),
            temp_dir,
            container_files: Mutex::new(HashMap::new()),
        }))
    }

    pub fn get() -> &'static Self {
        AUDIO_SERVICE.get().unwrap()
    }

    pub fn list_container(&self, source: AudioSourceRef) -> Result<AudioContainerInfo> {
        let source_path = self
            .pak_service
            .get_entry_path_by_hash(source.hash.hash_u64())?;
        let kind = audio_container_kind_from_path(&source_path)
            .ok_or_else(|| Error::AudioFileNotSupported(audio_type_error_hint(&source_path)))?;
        let container_path =
            self.ensure_container_file(source.hash.hash_u64(), &source_path, kind)?;

        list_container_from_file(source_path, kind, &container_path)
    }

    pub fn extract_wems(&self, options: AudioExtractBatchOptions) -> Result<Vec<PathBuf>> {
        let output_dir = options
            .output_dir
            .filter(|path| !path.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.temp_dir.clone());

        self.extract_wems_to_dir(options.source, &options.indices, &output_dir)
    }

    pub fn extract_wavs(&self, options: AudioExtractBatchOptions) -> Result<Vec<PathBuf>> {
        let output_dir = options
            .output_dir
            .filter(|path| !path.trim().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.temp_dir.clone());
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }

        let wems = self.extract_wems_to_dir(options.source, &options.indices, &self.temp_dir)?;
        wems.into_iter()
            .map(|wem_path| {
                let wav_file_name = wem_path
                    .with_extension("wav")
                    .file_name()
                    .ok_or_else(|| {
                        Error::Internal(format!("Invalid wem output path: {}", wem_path.display()))
                    })?
                    .to_owned();
                let wav_path = output_dir.join(Path::new(&wav_file_name));
                convert_wem_to_wav(&wem_path, &wav_path)?;
                Ok(wav_path)
            })
            .collect()
    }

    fn extract_wems_to_dir(
        &self,
        source: AudioSourceRef,
        indices: &[usize],
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let source_path = self
            .pak_service
            .get_entry_path_by_hash(source.hash.hash_u64())?;
        let kind = audio_container_kind_from_path(&source_path)
            .ok_or_else(|| Error::AudioFileNotSupported(audio_type_error_hint(&source_path)))?;
        let container_path =
            self.ensure_container_file(source.hash.hash_u64(), &source_path, kind)?;
        let wems = extract_wems_from_file(kind, &container_path, indices)?;

        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }

        let mut output_paths = Vec::with_capacity(wems.len());
        for wem in wems {
            let output_path = output_dir.join(build_temp_wem_file_name(
                source.hash.hash_u64(),
                wem.index,
                wem.wem_id,
            ));
            let mut file = File::create(&output_path)?;
            file.write_all(&wem.data)?;
            output_paths.push(output_path);
        }

        Ok(output_paths)
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

        if !self.temp_dir.exists() {
            std::fs::create_dir_all(&self.temp_dir)?;
        }

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

#[derive(Debug)]
struct ExtractedWem {
    index: usize,
    wem_id: u32,
    data: Vec<u8>,
}

fn list_container_from_file(
    source_path: String,
    kind: AudioContainerKind,
    container_path: impl AsRef<Path>,
) -> Result<AudioContainerInfo> {
    let mut reader = BufReader::new(File::open(container_path)?);
    list_container_from_reader(source_path, kind, &mut reader)
}

fn list_container_from_reader<R>(
    source_path: String,
    kind: AudioContainerKind,
    reader: &mut R,
) -> Result<AudioContainerInfo>
where
    R: Read + Seek,
{
    let entries = match kind {
        AudioContainerKind::Bnk => list_bnk_entries(reader)?,
        AudioContainerKind::Pck => list_pck_entries(reader)?,
    };

    Ok(AudioContainerInfo {
        source_path,
        container_kind: kind,
        entries,
    })
}

fn list_bnk_entries<R>(reader: &mut R) -> Result<Vec<AudioEntryInfo>>
where
    R: Read + Seek,
{
    let bnk = BnkRaw::read_from(reader)?;
    let entries = bnk.parse_didx()?;

    Ok(entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| AudioEntryInfo {
            index,
            wem_id: entry.id,
            offset: u64::from(entry.offset),
            size: u64::from(entry.length),
            language_id: None,
        })
        .collect())
}

fn list_pck_entries<R>(reader: R) -> Result<Vec<AudioEntryInfo>>
where
    R: Read + Seek,
{
    let pck = Pck::from_reader(reader)?;

    Ok(pck
        .header()
        .wem_entries
        .iter()
        .enumerate()
        .map(|(index, entry)| AudioEntryInfo {
            index,
            wem_id: entry.id,
            offset: u64::from(entry.offset),
            size: u64::from(entry.length),
            language_id: Some(entry.language_id),
        })
        .collect())
}

fn extract_wems_from_file(
    kind: AudioContainerKind,
    container_path: impl AsRef<Path>,
    indices: &[usize],
) -> Result<Vec<ExtractedWem>> {
    let file = File::open(container_path)?;
    let mut reader = BufReader::new(file);
    extract_wems_from_reader(kind, &mut reader, indices)
}

fn extract_wems_from_reader<R>(
    kind: AudioContainerKind,
    reader: &mut R,
    indices: &[usize],
) -> Result<Vec<ExtractedWem>>
where
    R: Read + Seek,
{
    match kind {
        AudioContainerKind::Bnk => extract_bnk_wems(reader, indices),
        AudioContainerKind::Pck => extract_pck_wems(reader, indices),
    }
}

fn extract_bnk_wems<R>(reader: &mut R, indices: &[usize]) -> Result<Vec<ExtractedWem>>
where
    R: Read + Seek,
{
    let bnk = BnkRaw::read_from(reader)?;
    let entries = bnk.parse_didx()?;
    let data_section = bnk
        .sections
        .iter()
        .find(|section| section.magic == *b"DATA")
        .ok_or(Error::SoundBnk(re_sound::bnk::BnkError::MissingSection(
            *b"DATA",
        )))?;

    indices
        .iter()
        .copied()
        .map(|index| {
            let entry = entries
                .get(index)
                .ok_or_else(|| Error::AudioEntryNotFound(index.to_string()))?;
            let start = entry.offset as usize;
            let end = start + entry.length as usize;
            let data = data_section.data.get(start..end).ok_or(Error::SoundBnk(
                re_sound::bnk::BnkError::BadDataSize {
                    name: format!("WEM {}", entry.id),
                    expected: end as u64,
                    got: data_section.data.len() as u64,
                    start: entry.offset as u64,
                },
            ))?;

            Ok(ExtractedWem {
                index,
                wem_id: entry.id,
                data: data.to_vec(),
            })
        })
        .collect()
}

fn extract_pck_wems<R>(reader: &mut R, indices: &[usize]) -> Result<Vec<ExtractedWem>>
where
    R: Read + Seek,
{
    let mut pck = Pck::from_reader(reader)?;

    indices
        .iter()
        .copied()
        .map(|index| {
            let wem_id = pck
                .header()
                .wem_entries
                .get(index)
                .map(|entry| entry.id)
                .ok_or_else(|| Error::AudioEntryNotFound(index.to_string()))?;
            let mut reader = pck
                .wem_reader(index)
                .ok_or_else(|| Error::AudioEntryNotFound(index.to_string()))?;
            let mut data = Vec::new();
            reader.read_to_end(&mut data)?;

            Ok(ExtractedWem {
                index,
                wem_id,
                data,
            })
        })
        .collect()
}

fn convert_wem_to_wav(wem_path: &Path, wav_path: &Path) -> Result<()> {
    if wav_path.exists() {
        return Ok(());
    }

    let cli_path = external_tools::find_vgmstream_cli().ok_or_else(|| {
        let expected = external_tools::vgmstream_cli_candidates()
            .into_iter()
            .next()
            .unwrap_or_else(|| external_tools::extension_dir("vgmstream"))
            .to_string_lossy()
            .to_string();
        Error::VgmstreamCliNotFound(expected)
    })?;

    if let Some(parent) = wav_path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    let mut command = Command::new(&cli_path);
    if let Some(parent) = cli_path.parent() {
        command.current_dir(parent);
        command.env(
            "DYLD_LIBRARY_PATH",
            build_child_library_path(parent, std::env::var_os("DYLD_LIBRARY_PATH")),
        );
    }

    let output = command
        .args(["-i", "-o"])
        .arg(wav_path)
        .arg(wem_path)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("exit status {}", output.status)
    };

    Err(Error::VgmstreamCliFailed(detail))
}

fn build_child_library_path(
    tool_dir: &Path,
    existing: Option<std::ffi::OsString>,
) -> std::ffi::OsString {
    let mut paths = vec![tool_dir.join("lib"), tool_dir.to_path_buf()];
    if let Some(existing) = existing {
        paths.extend(std::env::split_paths(&existing));
    }

    std::env::join_paths(paths).unwrap_or_else(|_| tool_dir.as_os_str().to_os_string())
}

fn audio_container_kind_from_path(path: &str) -> Option<AudioContainerKind> {
    let components = PathComponents::parse(path)?;
    match components.extension()?.to_ascii_lowercase().as_str() {
        "bnk" | "sbnk" => Some(AudioContainerKind::Bnk),
        "pck" | "spck" => Some(AudioContainerKind::Pck),
        _ => None,
    }
}

fn build_temp_wem_file_name(source_hash: u64, index: usize, wem_id: u32) -> String {
    format!("{source_hash:016X}-{index}-{wem_id}.wem")
}

fn build_temp_container_file_name(
    source_hash: u64,
    source_path: &str,
    kind: AudioContainerKind,
) -> String {
    let file_name = path_file_name(source_path);
    let extension = match kind {
        AudioContainerKind::Bnk => "bnk",
        AudioContainerKind::Pck => "pck",
    };
    format!(
        "{source_hash:016X}-{}.{}",
        sanitize_temp_file_name(file_name),
        extension
    )
}

fn audio_type_error_hint(path: &str) -> String {
    let file_name = path_file_name(path);
    file_name
        .rsplit('.')
        .next()
        .filter(|segment| !segment.is_empty())
        .unwrap_or(file_name)
        .to_string()
}

fn path_file_name(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
}

fn sanitize_temp_file_name(file_name: &str) -> String {
    file_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn audio_container_kind_supports_re_suffixes() {
        assert_eq!(
            audio_container_kind_from_path("foo.bnk"),
            Some(AudioContainerKind::Bnk)
        );
        assert_eq!(
            audio_container_kind_from_path("foo.sbnk.1.X64"),
            Some(AudioContainerKind::Bnk)
        );
        assert_eq!(
            audio_container_kind_from_path("foo.pck"),
            Some(AudioContainerKind::Pck)
        );
        assert_eq!(
            audio_container_kind_from_path("foo.spck.1.X64"),
            Some(AudioContainerKind::Pck)
        );
        assert_eq!(audio_container_kind_from_path("foo.tex.1.X64"), None);
    }

    #[test]
    fn audio_list_bnk_entries_and_extract_wem() {
        let wem_data = b"wem-data";
        let bytes = build_test_bnk(wem_data);
        let info = list_container_from_reader(
            "sound.sbnk.1.X64".to_string(),
            AudioContainerKind::Bnk,
            &mut Cursor::new(&bytes),
        )
        .unwrap();

        assert_eq!(info.container_kind, AudioContainerKind::Bnk);
        assert_eq!(info.entries.len(), 1);
        assert_eq!(
            info.entries[0],
            AudioEntryInfo {
                index: 0,
                wem_id: 7,
                offset: 0,
                size: wem_data.len() as u64,
                language_id: None,
            }
        );

        let wem = extract_wems_from_reader(AudioContainerKind::Bnk, &mut Cursor::new(&bytes), &[0])
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(wem.wem_id, 7);
        assert_eq!(wem.data, wem_data);
    }

    #[test]
    fn audio_list_pck_entries_and_extract_wem() {
        let wem_data = b"pck-wem";
        let bytes = build_test_pck(wem_data);
        let info = list_container_from_reader(
            "sound.spck.1.X64".to_string(),
            AudioContainerKind::Pck,
            &mut Cursor::new(&bytes),
        )
        .unwrap();

        assert_eq!(info.container_kind, AudioContainerKind::Pck);
        assert_eq!(info.entries.len(), 1);
        assert_eq!(
            info.entries[0],
            AudioEntryInfo {
                index: 0,
                wem_id: 11,
                offset: 56,
                size: wem_data.len() as u64,
                language_id: Some(3),
            }
        );

        let wem = extract_wems_from_reader(AudioContainerKind::Pck, &mut Cursor::new(&bytes), &[0])
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(wem.wem_id, 11);
        assert_eq!(wem.data, wem_data);
    }

    #[test]
    fn audio_extract_rejects_missing_entry() {
        let bytes = build_test_bnk(b"wem-data");
        let error =
            extract_wems_from_reader(AudioContainerKind::Bnk, &mut Cursor::new(&bytes), &[1])
                .unwrap_err();

        assert!(matches!(error, Error::AudioEntryNotFound(_)));
    }

    #[test]
    fn audio_extracts_selected_bnk_wems_in_order() {
        let bytes = build_test_bnk_many(&[(7, b"first".as_slice()), (8, b"second"), (9, b"third")]);
        let wems =
            extract_wems_from_reader(AudioContainerKind::Bnk, &mut Cursor::new(&bytes), &[2, 0])
                .unwrap();

        assert_eq!(wems.len(), 2);
        assert_eq!(wems[0].index, 2);
        assert_eq!(wems[0].wem_id, 9);
        assert_eq!(wems[0].data, b"third");
        assert_eq!(wems[1].index, 0);
        assert_eq!(wems[1].wem_id, 7);
        assert_eq!(wems[1].data, b"first");
    }

    #[test]
    fn audio_extracts_selected_pck_wems_in_order() {
        let bytes = build_test_pck_many(&[
            (11, 3, b"first".as_slice()),
            (12, 3, b"second"),
            (13, 4, b"third"),
        ]);
        let wems =
            extract_wems_from_reader(AudioContainerKind::Pck, &mut Cursor::new(&bytes), &[1, 2])
                .unwrap();

        assert_eq!(wems.len(), 2);
        assert_eq!(wems[0].index, 1);
        assert_eq!(wems[0].wem_id, 12);
        assert_eq!(wems[0].data, b"second");
        assert_eq!(wems[1].index, 2);
        assert_eq!(wems[1].wem_id, 13);
        assert_eq!(wems[1].data, b"third");
    }

    fn build_test_bnk(wem_data: &[u8]) -> Vec<u8> {
        build_test_bnk_many(&[(7, wem_data)])
    }

    fn build_test_bnk_many(wems: &[(u32, &[u8])]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"BKHD");
        write_u32(&mut bytes, 8);
        write_u32(&mut bytes, 1);
        write_u32(&mut bytes, 99);

        bytes.extend_from_slice(b"DIDX");
        write_u32(&mut bytes, (wems.len() * 12) as u32);
        let mut offset = 0u32;
        for (wem_id, data) in wems {
            write_u32(&mut bytes, *wem_id);
            write_u32(&mut bytes, offset);
            write_u32(&mut bytes, data.len() as u32);
            offset += data.len() as u32;
        }

        bytes.extend_from_slice(b"DATA");
        write_u32(&mut bytes, offset);
        for (_wem_id, data) in wems {
            bytes.extend_from_slice(data);
        }
        bytes
    }

    fn build_test_pck(wem_data: &[u8]) -> Vec<u8> {
        build_test_pck_many(&[(11, 3, wem_data)])
    }

    fn build_test_pck_many(wems: &[(u32, u32, &[u8])]) -> Vec<u8> {
        let wem_offset = (36 + wems.len() * 20) as u32;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"AKPK");
        write_u32(&mut bytes, wem_offset - 8);
        write_u32(&mut bytes, 0);
        write_u32(&mut bytes, 4);
        write_u32(&mut bytes, 0);
        write_u32(&mut bytes, (4 + wems.len() * 20) as u32);
        write_u32(&mut bytes, 0);

        write_u32(&mut bytes, 0);
        write_u32(&mut bytes, wems.len() as u32);
        let mut offset = wem_offset;
        for (wem_id, language_id, data) in wems {
            write_u32(&mut bytes, *wem_id);
            write_u32(&mut bytes, 1);
            write_u32(&mut bytes, data.len() as u32);
            write_u32(&mut bytes, offset);
            write_u32(&mut bytes, *language_id);
            offset += data.len() as u32;
        }

        assert_eq!(bytes.len(), wem_offset as usize);
        for (_wem_id, _language_id, data) in wems {
            bytes.extend_from_slice(data);
        }
        bytes
    }

    fn write_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}
