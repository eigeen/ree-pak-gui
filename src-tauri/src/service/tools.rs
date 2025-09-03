//! Tools service for path scanning and other utility functions

use anyhow::Result;
use parking_lot::Mutex;
use ree_path_searcher::{PathSearcher, SearchResult};
use std::{
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use crate::{channel::PathScanProgressChannel, command::PathScanOptions};

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

                let mut all_results = SearchResult {
                    found_paths: vec![],
                    unknown_paths: rustc_hash::FxHashSet::default(),
                };

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

                    all_results.found_paths.extend(result.found_paths);
                    all_results.unknown_paths.extend(result.unknown_paths);
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

                    all_results.found_paths.extend(result.found_paths);
                    all_results.unknown_paths.extend(result.unknown_paths);
                }

                // Sort and deduplicate results
                all_results.found_paths.sort_unstable_by(|(p, _), (q, _)| p.cmp(q));
                all_results.found_paths.dedup_by(|(p, _), (q, _)| p == q);

                // Convert results to the expected format
                let mut found_paths = Vec::new();
                for (_raw_path, indexes) in all_results.found_paths {
                    for index in indexes {
                        found_paths.push(index.full_path);
                    }
                }

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
}
