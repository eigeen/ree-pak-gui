//! Tools service for path scanning and other utility functions

use anyhow::{Context, Result};
use parking_lot::Mutex;
use ree_pak_core::filename::FileNameTable;
use ree_path_searcher::PathSearcher;
use std::{
    fs::File,
    io::BufReader,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use crate::{channel::PathScanProgressChannel, command::PathScanOptions, error::Error};

static TOOLS_SERVICE: OnceLock<ToolsService> = OnceLock::new();

pub struct ToolsService {
    work_thread: Mutex<Option<JoinHandle<()>>>,
    should_terminate: Arc<AtomicBool>,
}

impl ToolsService {
    pub fn get() -> &'static ToolsService {
        TOOLS_SERVICE.get_or_init(|| ToolsService {
            work_thread: Mutex::new(None),
            should_terminate: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn scan_paths(&self, options: &PathScanOptions, progress_channel: PathScanProgressChannel) -> Result<()> {
        // Check if there's already a running scan
        if let Some(handle) = &*self.work_thread.lock()
            && !handle.is_finished()
        {
            return Err(anyhow::anyhow!("Path scan already running"));
        }

        // Clone data for the thread
        let should_terminate = self.should_terminate.clone();
        let options = options.clone();
        let progress = progress_channel.clone();

        #[derive(Debug)]
        struct PathScanResult {
            success: bool,
            found_paths: Vec<String>,
            error: Option<String>,
        }

        // Spawn the scanning thread
        *self.work_thread.lock() = Some(thread::spawn(move || {
            let progress_clone = progress.clone();
            let main_fn = move || -> Result<PathScanResult> {
                // Start progress reporting
                progress_clone.file_start(0, 0);

                // Build PathSearcher
                let mut builder = PathSearcher::builder();

                // Load pak paths
                if !options.pak_files.is_empty() {
                    builder = builder.with_pak_paths(&options.pak_files);
                }

                let searcher = builder
                    .build()
                    .map_err(|e| anyhow::anyhow!("Failed to build path searcher: {}", e))?;

                let mut found_paths: Vec<String> = vec![];

                let total_files = options.dump_files.len() + options.pak_files.len();
                let mut current_file = 0;

                // Process dump files
                for dump_file in &options.dump_files {
                    // Check for termination
                    if should_terminate.load(Ordering::Relaxed) {
                        return Ok(PathScanResult {
                            success: false,
                            found_paths: vec![],
                            error: Some("Scan canceled".to_string()),
                        });
                    }

                    current_file += 1;
                    progress_clone.file_start(current_file, total_files as u32);

                    let result = searcher
                        .search_memory_dump(dump_file)
                        .map_err(|e| anyhow::anyhow!("Failed to scan memory dump {}: {}", dump_file, e))?;

                    found_paths.extend(
                        result
                            .found_paths
                            .into_iter()
                            .flat_map(|(_, p)| p.into_iter().map(|info| info.full_path)),
                    );
                }

                // Process PAK files if any
                if searcher.pak_file_count() > 0 {
                    // Check for termination
                    if should_terminate.load(Ordering::Relaxed) {
                        return Ok(PathScanResult {
                            success: false,
                            found_paths: vec![],
                            error: Some("Scan canceled".to_string()),
                        });
                    }

                    current_file += 1;
                    progress_clone.file_start(current_file, total_files as u32);

                    let result = searcher
                        .search_pak_files()
                        .map_err(|e| anyhow::anyhow!("Failed to scan pak files: {}", e))?;

                    found_paths.extend(
                        result
                            .found_paths
                            .into_iter()
                            .flat_map(|(_, p)| p.into_iter().map(|info| info.full_path)),
                    );
                }

                // check entry hash on loaded list file
                if let Some(list_path) = options.path_list_file {
                    let paths = Self::check_on_list_file(&list_path, &options.pak_files)?;
                    found_paths.extend(paths);
                }

                // Sort and deduplicate results
                found_paths.sort_unstable();
                found_paths.dedup();

                // Return results
                Ok(PathScanResult {
                    success: true,
                    found_paths,
                    error: None,
                })
            };

            match main_fn() {
                Ok(result) => {
                    if result.success {
                        progress.finish_ok(result.found_paths);
                    } else {
                        progress.finish_err(result.error.unwrap_or_default());
                    }
                }
                Err(e) => {
                    progress.finish_err(e.to_string());
                }
            }
        }));

        Ok(())
    }

    pub fn terminate_scan(&self) {
        if let Some(handle) = self.work_thread.lock().take() {
            self.should_terminate.store(true, Ordering::Relaxed);
            let _ = handle.join();
            self.should_terminate.store(false, Ordering::Relaxed);
        }
    }

    fn check_on_list_file(list_path: &str, pak_files: &[String]) -> Result<Vec<String>> {
        // open pak files
        let pak_metadatas = pak_files
            .iter()
            .map(|path| {
                let file = File::open(path)
                    .map_err(|e| Error::FileIO {
                        path: path.to_string(),
                        source: e,
                    })
                    .context("Failed to open pak file")?;

                let mut reader = BufReader::new(file);
                let metadata = ree_pak_core::read::read_metadata(&mut reader)?;
                Ok(metadata)
            })
            .collect::<Result<Vec<_>>>()?;

        // load list file
        let list_file = FileNameTable::from_list_file(list_path)?;

        let mut found_paths = vec![];
        for metadata in pak_metadatas {
            for entry in metadata.entries() {
                if let Some(path) = list_file.get_file_name(entry.hash()) {
                    found_paths.push(path.to_string().unwrap());
                }
            }
        }

        Ok(found_paths)
    }
}
