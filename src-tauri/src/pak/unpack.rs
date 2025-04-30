use std::{
    collections::HashSet,
    fs::OpenOptions,
    hash::BuildHasherDefault,
    io::{BufRead, Read, Seek, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

use nohash::NoHashHasher;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ree_pak_core::{
    filename::FileNameTable,
    read::{archive::PakArchiveReader, entry::PakEntryReader},
};

use crate::channel::ProgressChannel;

use super::{ExtractOptions, Pak};

pub fn unpack_parallel_error_continue<R>(
    pak: &mut Pak<R>,
    file_name_table: &FileNameTable,
    options: &ExtractOptions,
    progress: ProgressChannel,
) -> anyhow::Result<()>
where
    R: Read + Seek + Send,
{
    let mut target_files: HashSet<u64, BuildHasherDefault<NoHashHasher<u64>>> = HashSet::default();
    for info in options.extract_files.iter() {
        if info.belongs_to == pak.id {
            target_files.insert(info.hash.hash_u64());
        }
    }
    if pak.reader.is_none() {
        return Err(anyhow::anyhow!("Pak reader is not set"));
    }

    let archive_reader = Mutex::new(PakArchiveReader::new(pak.reader.take().unwrap(), &pak.archive));

    let output_path = Path::new(&options.output_path);

    // extract files
    let _results: Vec<anyhow::Result<()>> = pak
        .archive
        .entries()
        .par_iter()
        .map(|entry| -> anyhow::Result<()> {
            if !(options.extract_all || target_files.contains(&entry.hash())) {
                return Ok(());
            }

            // get entry reader
            let entry_reader = {
                let mut r = archive_reader.lock().unwrap();
                (*r).owned_entry_reader(entry.clone())?
            };
            // output file path
            let file_relative_path: PathBuf = file_name_table
                .get_file_name(entry.hash())
                .map(|fname| fname.get_name().to_string())
                .unwrap_or_else(|| format!("_Unknown/{:08X}", entry.hash()))
                .into();
            let output_path = output_path.join(&file_relative_path);

            progress.file_start(file_relative_path.to_string_lossy().to_string(), entry.hash());
            let result = process_entry(entry_reader, output_path, true);
            if let Err(e) = &result {
                log::error!(
                    "Error processing entry: {}. Path: {:?}",
                    e,
                    file_name_table.get_file_name(entry.hash()).unwrap(),
                );
                log::debug!("Entry: {:?}", entry);
                progress.file_done(entry.hash(), Some(e.to_string()));
            } else {
                progress.file_done(entry.hash(), None);
            };

            result
        })
        .collect();

    pak.reader.replace(archive_reader.into_inner().unwrap().into_inner());

    Ok(())
}

fn process_entry<R>(mut entry_reader: PakEntryReader<R>, output_path: PathBuf, r#override: bool) -> anyhow::Result<()>
where
    R: BufRead + Seek,
{
    let file_dir = output_path.parent().unwrap();

    if !file_dir.exists() {
        std::fs::create_dir_all(file_dir)?;
    }

    let mut data = vec![];
    std::io::copy(&mut entry_reader, &mut data)?;

    let mut file = if r#override {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&output_path)?
    } else {
        OpenOptions::new().create_new(true).write(true).open(&output_path)?
    };
    file.write_all(&data)?;

    // guess unknown file extension
    if output_path.extension().is_none() {
        if let Some(ext) = entry_reader.determine_extension() {
            let new_path = output_path.with_extension(ext);
            std::fs::rename(output_path, new_path)?;
        }
    }

    Ok(())
}
