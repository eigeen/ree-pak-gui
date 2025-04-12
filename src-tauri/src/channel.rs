use std::sync::{Arc, atomic::AtomicU32};

use serde::Serialize;
use tauri::ipc::Channel;

/// Work progress event.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
#[allow(clippy::enum_variant_names)]
pub enum WorkProgressEvent {
    #[serde(rename_all = "camelCase")]
    Start {
        file_count: u32,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        finished_count: u32,
    },
    Finished,
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

    pub fn start(&self, file_count: u32) {
        if let Err(e) = self.channel.send(WorkProgressEvent::Start { file_count }) {
            log::error!("Failed to send start event: {}", e);
        }
    }

    pub fn inc_finished(&self) {
        let finished_count = self.finished_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if let Err(e) = self.channel.send(WorkProgressEvent::Progress {
            finished_count: finished_count + 1,
        }) {
            log::error!("Failed to send progress event: {}", e);
        }
    }

    pub fn finished(&self) {
        if let Err(e) = self.channel.send(WorkProgressEvent::Finished) {
            log::error!("Failed to send finished event: {}", e);
        }
    }
}
