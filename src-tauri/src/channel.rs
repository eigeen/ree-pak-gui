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
pub enum WorkProgressEvent<T> {
    /// Work start.
    #[serde(rename_all = "camelCase")]
    WorkStart { count: u32 },
    /// File extraction done.
    FileDone(T),
    /// Work finished.
    WorkFinished,
    /// Error.
    #[serde(rename_all = "camelCase")]
    Error { error: String },
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnpackProgressData {
    path: String,
    hash: JsSafeHash,
    finish_count: u32,
}

pub type UnpackProgressChannel = UnpackProgressChannelImpl<UnpackProgressData>;
pub type UnpackProgressChannelInner = Channel<WorkProgressEvent<UnpackProgressData>>;

#[derive(Clone)]
pub struct UnpackProgressChannelImpl<T> {
    channel: Channel<WorkProgressEvent<T>>,
    finish_count: Arc<AtomicU32>,
    steady_tick: Duration,
    last_tick: Arc<Mutex<Instant>>,
}

impl UnpackProgressChannelImpl<UnpackProgressData> {
    pub fn new(channel: Channel<WorkProgressEvent<UnpackProgressData>>) -> Self {
        let steady_tick = Duration::from_millis(100);
        Self {
            channel,
            finish_count: Arc::new(AtomicU32::new(0)),
            steady_tick,
            last_tick: Arc::new(Mutex::new(Instant::now() - steady_tick)),
        }
    }

    pub fn work_start(&self, count: u32) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkStart { count }) {
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

        if let Err(e) = self.channel.send(WorkProgressEvent::FileDone(UnpackProgressData {
            path: path.to_string(),
            hash: JsSafeHash::from_u64(hash),
            finish_count,
        })) {
            log::error!("Failed to send file done event: {}", e);
        }
    }

    pub fn work_finished(&self) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkFinished) {
            log::error!("Failed to send work finished event: {}", e);
        }
    }
}

// Pack progress

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackProgressData {
    path: String,
    finish_count: u32,
}

pub type PackProgressChannel = PackProgressChannelImpl<PackProgressData>;
pub type PackProgressChannelInner = Channel<WorkProgressEvent<PackProgressData>>;

#[derive(Clone)]
pub struct PackProgressChannelImpl<T> {
    channel: Channel<WorkProgressEvent<T>>,
    finish_count: Arc<AtomicU32>,
}

impl PackProgressChannelImpl<PackProgressData> {
    pub fn new(channel: Channel<WorkProgressEvent<PackProgressData>>) -> Self {
        Self {
            channel,
            finish_count: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn work_start(&self, count: u32) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkStart { count }) {
            log::error!("Failed to send work start event: {}", e);
        }
    }

    pub fn file_done(&self, path: &str) {
        let finish_count = self.finish_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

        if let Err(e) = self.channel.send(WorkProgressEvent::FileDone(PackProgressData {
            path: path.to_string(),
            finish_count,
        })) {
            log::error!("Failed to send file done event: {}", e);
        }
    }

    pub fn work_finished(&self) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkFinished) {
            log::error!("Failed to send work finished event: {}", e);
        }
    }

    pub fn error(&self, error: String) {
        if let Err(e) = self.channel.send(WorkProgressEvent::Error { error }) {
            log::error!("Failed to send error event: {}", e);
        }
    }
}
