// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::OnceLock;

use tauri::{AppHandle, Manager};

mod channel;
mod command;
mod common;
mod error;
mod event;
mod filename;
mod logger;
mod macros;
mod pak;
mod usecase;
mod utility;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

fn panic_hook(info: &std::panic::PanicHookInfo) {
    #[cfg(target_os = "windows")]
    utility::message_box_error(&format!("panic occurred: {:#}", info));
    #[cfg(not(target_os = "windows"))]
    eprintln!("panic occurred: {:#}", info);
}

fn main() {
    std::panic::set_hook(Box::new(panic_hook));
    logger::Logger::init();

    tauri::Builder::default()
        .setup(|app| {
            let _ = APP_HANDLE.set(app.handle().clone());
            let main_window = app.get_webview_window("main").unwrap();
            main_window
                .set_title(&format!("REE Pak GUI - v{}", env!("CARGO_PKG_VERSION")))
                .unwrap();
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            command::pak_clear_all,
            command::pak_list_all,
            command::pak_open,
            command::pak_close,
            command::pak_order,
            command::pak_get_info,
            command::pak_read_file_tree,
            command::pak_read_file_tree_optimized,
            command::pak_extract_all,
            command::file_table_get_list,
            command::file_table_load,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
