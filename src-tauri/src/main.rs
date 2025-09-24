// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, sync::OnceLock};

use pak::group::PakGroup;
use service::pak::PakService;
use tauri::{AppHandle, Manager};

use crate::service::preview::PreviewService;

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

const LOCAL_DIR_PATH: &str = "ree-pak-tools";
const TEMP_DIR_NAME: &str = "temp";

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

fn panic_hook(info: &std::panic::PanicHookInfo) {
    #[cfg(target_os = "windows")]
    utility::message_box_error(&format!("panic occurred: {:#}", info));
    #[cfg(not(target_os = "windows"))]
    eprintln!("panic occurred: {:#}", info);
    std::process::exit(1);
}

// Clean temp files.
fn clean_temp_files() {
    let temp_dir = get_local_dir().join(TEMP_DIR_NAME);
    if temp_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
            log::warn!("Failed to clean temp files: {}", e);
        } else {
            log::info!("Temp files cleaned: {:?}", temp_dir);
        }
    }
}

fn get_local_dir() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let local_dir = exe_dir.join(LOCAL_DIR_PATH);
    if !local_dir.exists() {
        std::fs::create_dir_all(&local_dir).unwrap();
    }

    local_dir
}

fn main() {
    std::panic::set_hook(Box::new(panic_hook));
    logger::Logger::init();

    // initialize services
    let _ = PakService::initialize(PakGroup::new());
    let _ = PreviewService::initialize();

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let _ = APP_HANDLE.set(app.handle().clone());
            let main_window = app.get_webview_window("main").unwrap();
            main_window
                .set_title(&format!("REE Pak Tool - v{}", env!("CARGO_PKG_VERSION")))
                .unwrap();
            Ok(())
        })
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
            command::file_table_push_paths,
            command::get_preview_file,
            command::get_exe_path,
            command::get_compile_info,
            command::perform_update,
            command::zip_extract_file,
            command::murmur32,
            command::murmur32_utf16,
            command::tools_scan_paths,
            command::tools_terminate_scan,
        ])
        .on_window_event(|window, event| {
            // Clean temp files when main window is closed.
            if let tauri::WindowEvent::CloseRequested { .. } = event
                && window.label() == "main"
            {
                clean_temp_files();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
