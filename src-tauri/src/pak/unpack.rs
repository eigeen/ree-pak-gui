use std::{
    collections::{HashMap, HashSet},
    fs::{File, OpenOptions},
    hash::BuildHasherDefault,
    io::{BufReader, Read, Seek},
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

use nohash::NoHashHasher;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ree_pak_core::{filename::FileNameTable, pak::PakEntry, read::io::archive::PakArchiveReader};

use super::{ExtractOptions, Pak};

pub fn unpack_parallel_error_continue<R>(
    pak: &mut Pak<R>,
    file_name_table: &FileNameTable,
    options: &ExtractOptions,
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
    // let bar = ProgressBar::new(archive.entries().len() as u64);
    // bar.set_style(
    //     ProgressStyle::default_bar().template("{pos}/{len} files written {wide_bar} elapsed: {elapsed} eta: {eta}")?,
    // );
    // bar.enable_steady_tick(Duration::from_millis(100));
    // bar.println(format!("Output directory: `{}`", output_path.display()));
    let results: Vec<anyhow::Result<()>> = pak
        .archive
        .entries()
        .par_iter()
        .map(|entry| -> anyhow::Result<()> {
            if options.extract_all || target_files.contains(&entry.hash()) {
                let result = process_entry(entry, file_name_table, output_path, &archive_reader, true);
                if let Err(e) = &result {
                    // bar.println(format!("Error processing entry: {}\nEntry: {:?}", e, entry));
                };
                return result;
            }
            Ok(())
        })
        .collect();

    // bar.finish();

    // if !results.is_empty() {
    //     println!("Done with {} errors", results.len());
    // } else {
    //     println!("Done.");
    // }

    pak.reader.replace(archive_reader.into_inner().unwrap().into_inner());

    Ok(())
}

fn process_entry<R>(
    entry: &PakEntry,
    file_name_table: &FileNameTable,
    output_path: &Path,
    archive_reader: &Mutex<PakArchiveReader<R>>,
    r#override: bool,
) -> anyhow::Result<()>
where
    R: Read + Seek,
{
    let mut r = archive_reader.lock().unwrap();
    let mut entry_reader = (*r).owned_entry_reader(entry.clone())?;
    drop(r);

    // output file path
    let file_relative_path: PathBuf = file_name_table
        .get_file_name(entry.hash())
        .map(|fname| fname.get_name().to_string())
        .unwrap_or_else(|| format!("_Unknown/{:08X}", entry.hash()))
        .into();
    let filepath = output_path.join(file_relative_path);
    let filedir = filepath.parent().unwrap();

    if !filedir.exists() {
        std::fs::create_dir_all(filedir)?;
    }

    let mut file = if r#override {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&filepath)?
    } else {
        OpenOptions::new().create_new(true).write(true).open(&filepath)?
    };
    std::io::copy(&mut entry_reader, &mut file)?;

    // guess unknown file extension
    if filepath.extension().is_none() {
        if let Some(ext) = entry_reader.determine_extension() {
            let new_path = filepath.with_extension(ext);
            std::fs::rename(filepath, new_path)?;
        }
    }

    Ok(())
}
