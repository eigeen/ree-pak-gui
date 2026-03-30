use std::{
    sync::{Arc, atomic::AtomicU32},
    time::{Duration, Instant},
};

use parking_lot::Mutex;
use serde::Serialize;
use tauri::ipc::Channel;

use crate::common::JsSafeHash;

const PROGRESS_EVENT_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone)]
struct ProgressThrottle {
    steady_tick: Duration,
    last_tick: Arc<Mutex<Instant>>,
}

impl ProgressThrottle {
    fn new(steady_tick: Duration) -> Self {
        Self {
            steady_tick,
            last_tick: Arc::new(Mutex::new(Instant::now() - steady_tick)),
        }
    }

    fn should_emit(&self) -> bool {
        let mut last_tick = self.last_tick.lock();
        if last_tick.elapsed() < self.steady_tick {
            return false;
        }
        *last_tick = Instant::now();
        true
    }
}

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
    WorkFinished(Option<T>),
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
    throttle: ProgressThrottle,
}

impl UnpackProgressChannelImpl<UnpackProgressData> {
    pub fn new(channel: Channel<WorkProgressEvent<UnpackProgressData>>) -> Self {
        Self {
            channel,
            finish_count: Arc::new(AtomicU32::new(0)),
            throttle: ProgressThrottle::new(PROGRESS_EVENT_INTERVAL),
        }
    }

    pub fn work_start(&self, count: u32) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkStart { count }) {
            log::error!("Failed to send work start event: {}", e);
        }
    }

    pub fn file_done(&self, path: &str, hash: u64, _err_msg: Option<String>) {
        let finish_count = self
            .finish_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;

        if !self.throttle.should_emit() {
            return;
        }

        if let Err(e) = self
            .channel
            .send(WorkProgressEvent::FileDone(UnpackProgressData {
                path: path.to_string(),
                hash: JsSafeHash::from_u64(hash),
                finish_count,
            }))
        {
            log::error!("Failed to send file done event: {}", e);
        }
    }

    pub fn work_finished(&self) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkFinished(None)) {
            log::error!("Failed to send work finished event: {}", e);
        }
    }

    pub fn error(&self, error: String) {
        if let Err(e) = self.channel.send(WorkProgressEvent::Error { error }) {
            log::error!("Failed to send work error event: {}", e);
        }
    }
}

// Texture export progress

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureExportProgressData {
    path: String,
    finish_count: u32,
}

pub type TextureExportProgressChannel = TextureExportProgressChannelImpl<TextureExportProgressData>;
pub type TextureExportProgressChannelInner = Channel<WorkProgressEvent<TextureExportProgressData>>;

#[derive(Clone)]
pub struct TextureExportProgressChannelImpl<T> {
    channel: Channel<WorkProgressEvent<T>>,
    finish_count: Arc<AtomicU32>,
    throttle: ProgressThrottle,
}

impl TextureExportProgressChannelImpl<TextureExportProgressData> {
    pub fn new(channel: Channel<WorkProgressEvent<TextureExportProgressData>>) -> Self {
        Self {
            channel,
            finish_count: Arc::new(AtomicU32::new(0)),
            throttle: ProgressThrottle::new(PROGRESS_EVENT_INTERVAL),
        }
    }

    pub fn work_start(&self, count: u32) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkStart { count }) {
            log::error!("Failed to send texture export start event: {}", e);
        }
    }

    pub fn file_done(&self, path: &str) {
        let finish_count = self
            .finish_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;

        if !self.throttle.should_emit() {
            return;
        }

        if let Err(e) = self
            .channel
            .send(WorkProgressEvent::FileDone(TextureExportProgressData {
                path: path.to_string(),
                finish_count,
            }))
        {
            log::error!("Failed to send texture export file done event: {}", e);
        }
    }

    pub fn work_finished(&self) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkFinished(None)) {
            log::error!("Failed to send texture export finished event: {}", e);
        }
    }

    pub fn error(&self, error: String) {
        if let Err(e) = self.channel.send(WorkProgressEvent::Error { error }) {
            log::error!("Failed to send texture export error event: {}", e);
        }
    }
}

// Pack progress

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackProgressData {
    path: String,
    finish_count: u32,
    tree: Option<PackedFileTree>,
}

pub type PackProgressChannel = PackProgressChannelImpl<PackProgressData>;
pub type PackProgressChannelInner = Channel<WorkProgressEvent<PackProgressData>>;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackedFileTree {
    paks: Vec<PackedPak>,
}

impl PackedFileTree {
    pub fn new(paks: Vec<PackedPak>) -> Self {
        Self { paks }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackedPak {
    path: String,
    files: Vec<PackedFile>,
}

impl PackedPak {
    pub fn new(path: String, files: Vec<PackedFile>) -> Self {
        Self { path, files }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackedFile {
    path: String,
    hash: JsSafeHash,
    size: u64,
}

impl PackedFile {
    pub fn new(path: String, hash: JsSafeHash, size: u64) -> Self {
        Self { path, hash, size }
    }
}

#[derive(Clone)]
pub struct PackProgressChannelImpl<T> {
    channel: Channel<WorkProgressEvent<T>>,
    finish_count: Arc<AtomicU32>,
    throttle: ProgressThrottle,
}

impl PackProgressChannelImpl<PackProgressData> {
    pub fn new(channel: Channel<WorkProgressEvent<PackProgressData>>) -> Self {
        Self {
            channel,
            finish_count: Arc::new(AtomicU32::new(0)),
            throttle: ProgressThrottle::new(PROGRESS_EVENT_INTERVAL),
        }
    }

    pub fn work_start(&self, count: u32) {
        if let Err(e) = self.channel.send(WorkProgressEvent::WorkStart { count }) {
            log::error!("Failed to send work start event: {}", e);
        }
    }

    pub fn file_done(&self, path: &str) {
        let finish_count = self
            .finish_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;

        if !self.check_tick() {
            return;
        }

        if let Err(e) = self
            .channel
            .send(WorkProgressEvent::FileDone(PackProgressData {
                path: path.to_string(),
                finish_count,
                tree: None,
            }))
        {
            log::error!("Failed to send file done event: {}", e);
        }
    }

    pub fn work_finished(&self, tree: PackedFileTree) {
        if let Err(e) = self
            .channel
            .send(WorkProgressEvent::WorkFinished(Some(PackProgressData {
                path: "".to_string(),
                finish_count: 0,
                tree: Some(tree),
            })))
        {
            log::error!("Failed to send work finished event: {}", e);
        }
    }

    pub fn error(&self, error: String) {
        if let Err(e) = self.channel.send(WorkProgressEvent::Error { error }) {
            log::error!("Failed to send error event: {}", e);
        }
    }

    fn check_tick(&self) -> bool {
        self.throttle.should_emit()
    }
}

// Path scan progress

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum PathScanProgressEvent {
    #[serde(rename_all = "camelCase")]
    StartFile { current: u32, total: u32 },
    #[serde(rename_all = "camelCase")]
    Finish {
        success: bool,
        found_paths: Vec<String>,
        error: Option<String>,
    },
}

pub type PathScanProgressChannel = PathScanProgressChannelImpl;
pub type PathScanProgressChannelInner = Channel<PathScanProgressEvent>;

#[derive(Clone)]
pub struct PathScanProgressChannelImpl {
    channel: Channel<PathScanProgressEvent>,
}

impl PathScanProgressChannelImpl {
    pub fn new(channel: Channel<PathScanProgressEvent>) -> Self {
        Self { channel }
    }

    pub fn file_start(&self, current: u32, total: u32) {
        if let Err(e) = self
            .channel
            .send(PathScanProgressEvent::StartFile { current, total })
        {
            log::error!("Failed to send path scan progress event: {}", e);
        }
    }

    pub fn finish_ok(&self, found_paths: Vec<String>) {
        if let Err(e) = self.channel.send(PathScanProgressEvent::Finish {
            success: true,
            found_paths,
            error: None,
        }) {
            log::error!("Failed to send path scan progress event: {}", e);
        }
    }

    pub fn finish_err(&self, error: String) {
        if let Err(e) = self.channel.send(PathScanProgressEvent::Finish {
            success: false,
            found_paths: Vec::new(),
            error: Some(error),
        }) {
            log::error!("Failed to send path scan progress event: {}", e);
        }
    }
}
