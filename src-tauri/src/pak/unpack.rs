use std::{
    collections::{HashMap, HashSet},
    fs::{File, OpenOptions},
    hash::BuildHasherDefault,
    io::{BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

use nohash::NoHashHasher;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ree_pak_core::{filename::FileNameTable, pak::PakEntry, read::archive::PakArchiveReader};

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
                    println!(
                        "Error processing entry: {}. Path: {:?}\nEntry: {:?}",
                        e,
                        file_name_table.get_file_name(entry.hash()).unwrap(),
                        entry
                    )
                    // bar.println(format!("Error processing entry: {}\nEntry: {:?}", e, entry));
                };
                return result;
            }
            Ok(())
        })
        .collect();

    // bar.finish();

    if !results.is_empty() {
        let errors = results.iter().filter(|r| r.is_err()).collect::<Vec<_>>();
        println!("Done with {} errors", errors.len());
        if errors.len() < 30 {
            println!("Errors: {:?}", errors);
        } else {
            println!("Too many errors to display.");
        }
    } else {
        println!("Done.");
    }

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
    let file_path = output_path.join(file_relative_path);
    let file_dir = file_path.parent().unwrap();

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
            .open(&file_path)?
    } else {
        OpenOptions::new().create_new(true).write(true).open(&file_path)?
    };
    file.write_all(&data)?;

    // guess unknown file extension
    if file_path.extension().is_none() {
        if let Some(ext) = entry_reader.determine_extension() {
            let new_path = file_path.with_extension(ext);
            std::fs::rename(file_path, new_path)?;
        }
    }

    Ok(())
}
