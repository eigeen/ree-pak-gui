// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::OnceLock;

use pak::group::PakGroup;
use service::pak::PakService;
use tauri::{AppHandle, Manager};

mod channel;
mod command;
mod common;
mod error;
mod event;
mod logger;
mod macros;
mod pak;
mod service;
mod utility;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

fn panic_hook(info: &std::panic::PanicHookInfo) {
    #[cfg(target_os = "windows")]
    utility::message_box_error(&format!("panic occurred: {:#}", info));
    #[cfg(not(target_os = "windows"))]
    eprintln!("panic occurred: {:#}", info);
    std::process::exit(1);
}

fn main() {
    std::panic::set_hook(Box::new(panic_hook));
    logger::Logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let _ = APP_HANDLE.set(app.handle().clone());
            let main_window = app.get_webview_window("main").unwrap();
            main_window
                .set_title(&format!("REE Pak Tool - v{}", env!("CARGO_PKG_VERSION")))
                .unwrap();
            Ok(())
        })
        .manage(PakService::new(PakGroup::new()))
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
            command::pak_terminate_extraction,
            command::pak_get_header,
            command::pak_pack,
            command::pak_terminate_pack,
            command::file_table_load,
            command::get_exe_path,
            command::get_compile_info,
            command::perform_update,
            command::zip_extract_file,
            command::murmur32,
            command::murmur32_utf16,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
