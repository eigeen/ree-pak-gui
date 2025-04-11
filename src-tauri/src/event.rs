use std::fmt::Display;

use serde::Serialize;
use tauri::Emitter;

use crate::APP_HANDLE;

/// System event.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum SystemEvent {
    Log { level: LogLevel, message: String },
}

/// Log level.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

pub struct SystemEventSender;

impl SystemEventSender {
    pub fn send(&self, event: SystemEvent) {
        let Some(app) = APP_HANDLE.get() else {
            return;
        };
        if let Err(e) = app.emit("system", event) {
            eprintln!("Failed to send system event: {}", e);
        }
    }

    pub fn log(&self, level: LogLevel, message: impl Display) {
        self.send(SystemEvent::Log {
            level,
            message: message.to_string(),
        });
    }
}
