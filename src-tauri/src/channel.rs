use std::sync::{Arc, atomic::AtomicU32};

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
    /// File extraction start.
    #[serde(rename_all = "camelCase")]
    FileStart { path: String, hash: JsSafeHash },
    /// File extraction done.
    #[serde(rename_all = "camelCase")]
    FileDone {
        hash: JsSafeHash,
        finish_count: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        err_msg: Option<String>,
    },
    /// Work finished.
    WorkFinished,
}

#[derive(Clone)]
pub struct ProgressChannel {
    channel: Channel<WorkProgressEvent>,
    finished_count: Arc<AtomicU32>,
}

impl ProgressChannel {
    pub fn new(channel: Channel<WorkProgressEvent>) -> Self {
        Self {
            channel,
            finished_count: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn work_start(&self, file_count: u32) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkStart { file_count }) {
            log::error!("Failed to send work start event: {}", e);
        }
    }

    pub fn file_start(&self, path: String, hash: u64) {
        if let Err(e) = self.channel.send(WorkProgressEvent::FileStart {
            path,
            hash: JsSafeHash::from_u64(hash),
        }) {
            log::error!("Failed to send file start event: {}", e);
        }
    }

    pub fn file_done(&self, hash: u64, err_msg: Option<String>) {
        let finish_count = self.finished_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if let Err(e) = self.channel.send(WorkProgressEvent::FileDone {
            hash: JsSafeHash::from_u64(hash),
            finish_count,
            err_msg,
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
