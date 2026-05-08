use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    path::Path,
};

use re_sound::{bnk::BnkRaw, pck::Pck};

use crate::{
    error::{Error, Result},
    path_components::PathComponents,
};

use super::{AudioContainerInfo, AudioContainerKind, AudioEntryInfo};

#[derive(Debug)]
pub(super) struct ExtractedWem {
    pub index: usize,
    pub wem_id: u32,
    pub data: Vec<u8>,
}

pub(super) fn list_container_from_file(
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

    Ok(bnk
        .parse_didx()?
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

pub(super) fn extract_wems_from_file(
    kind: AudioContainerKind,
    container_path: impl AsRef<Path>,
    indices: &[usize],
) -> Result<Vec<ExtractedWem>> {
    let file = File::open(container_path)?;
    extract_wems_from_reader(kind, &mut BufReader::new(file), indices)
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
        .map(|index| extract_bnk_wem(index, &entries, data_section))
        .collect()
}

fn extract_bnk_wem(
    index: usize,
    entries: &[re_sound::bnk::DidxEntry],
    data_section: &re_sound::bnk::RawSection,
) -> Result<ExtractedWem> {
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
}

fn extract_pck_wems<R>(reader: &mut R, indices: &[usize]) -> Result<Vec<ExtractedWem>>
where
    R: Read + Seek,
{
    let mut pck = Pck::from_reader(reader)?;

    indices
        .iter()
        .copied()
        .map(|index| extract_pck_wem(&mut pck, index))
        .collect()
}

fn extract_pck_wem<R>(pck: &mut Pck<R>, index: usize) -> Result<ExtractedWem>
where
    R: Read + Seek,
{
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
}

pub(super) fn audio_container_kind_from_path(path: &str) -> Option<AudioContainerKind> {
    let components = PathComponents::parse(path)?;
    match components.extension()?.to_ascii_lowercase().as_str() {
        "bnk" | "sbnk" => Some(AudioContainerKind::Bnk),
        "pck" | "spck" => Some(AudioContainerKind::Pck),
        _ => None,
    }
}

pub(super) fn build_temp_wem_file_name(source_hash: u64, index: usize, wem_id: u32) -> String {
    format!("{source_hash:016X}-{index}-{wem_id}.wem")
}

pub(super) fn build_temp_container_file_name(
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

pub(super) fn audio_type_error_hint(path: &str) -> String {
    path_file_name(path)
        .rsplit('.')
        .next()
        .filter(|segment| !segment.is_empty())
        .unwrap_or_else(|| path_file_name(path))
        .to_string()
}

fn path_file_name(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
}

fn sanitize_temp_file_name(file_name: &str) -> String {
    file_name.chars().map(sanitize_temp_file_char).collect()
}

fn sanitize_temp_file_char(ch: char) -> char {
    if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
        ch
    } else {
        '_'
    }
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
        let bytes = build_test_bnk_many(&[(7, b"first"), (8, b"second"), (9, b"third")]);
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
        let bytes =
            build_test_pck_many(&[(11, 3, b"first"), (12, 3, b"second"), (13, 4, b"third")]);
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
