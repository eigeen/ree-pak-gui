use std::{fmt::Display, sync::LazyLock};

use serde::Serialize;
use tauri::ipc::Channel;

/// Work event.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
#[allow(clippy::enum_variant_names)]
enum WorkEvent {
    UnpackStart { file_count: u32 },
    UnpackProgress { finished_count: u32 },
    UnpackFinished,
}
