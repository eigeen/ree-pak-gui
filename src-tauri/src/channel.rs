use std::{
    sync::{Arc, atomic::AtomicU32},
    time::{Duration, Instant},
};

use parking_lot::Mutex;
use serde::Serialize;
use tauri::ipc::Channel;

use crate::common::JsSafeHash;

/// Work progress event.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
#[allow(clippy::enum_variant_names)]
pub enum WorkProgressEvent {
    /// Work start.
    #[serde(rename_all = "camelCase")]
    WorkStart { file_count: u32 },
    /// File extraction done.
    #[serde(rename_all = "camelCase")]
    FileDone {
        path: String,
        hash: JsSafeHash,
        finish_count: u32,
    },
    /// Work finished.
    WorkFinished,
}

#[derive(Clone)]
pub struct ProgressChannel {
    channel: Channel<WorkProgressEvent>,
    finish_count: Arc<AtomicU32>,
    steady_tick: Duration,
    last_tick: Arc<Mutex<Instant>>,
}

impl ProgressChannel {
    pub fn new(channel: Channel<WorkProgressEvent>) -> Self {
        let steady_tick = Duration::from_millis(100);
        Self {
            channel,
            finish_count: Arc::new(AtomicU32::new(0)),
            steady_tick,
            last_tick: Arc::new(Mutex::new(Instant::now() - steady_tick)),
        }
    }

    pub fn work_start(&self, file_count: u32) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkStart { file_count }) {
            log::error!("Failed to send work start event: {}", e);
        }
    }

    pub fn file_done(&self, path: &str, hash: u64, _err_msg: Option<String>) {
        let finish_count = self.finish_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

        let mut last_tick = self.last_tick.lock();
        if last_tick.elapsed() < self.steady_tick {
            return;
        }
        *last_tick = Instant::now();

        if let Err(e) = self.channel.send(WorkProgressEvent::FileDone {
            path: path.to_string(),
            hash: JsSafeHash::from_u64(hash),
            finish_count,
        }) {
            log::error!("Failed to send file done event: {}", e);
        }
    }

    pub fn work_finished(&self) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkFinished) {
            log::error!("Failed to send work finished event: {}", e);
        }
    }
}
