// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    path::PathBuf,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use pak::group::PakGroup;
use service::pak::PakService;
use tauri::{AppHandle, Manager};

use crate::service::{
    audio::AudioService, model_insight::ModelInsightService, preview::PreviewService,
};

mod channel;
mod command;
mod common;
mod error;
mod event;
mod external_tools;
mod logger;
mod macros;
mod pak;
mod path_components;
mod service;
mod utility;

const LOCAL_DIR_PATH: &str = "ree-pak-tools";
const TEMP_DIR_NAME: &str = "temp";
const RELEASE_PREVIEW_REFERENCES_SCRIPT: &str = r#"
window.dispatchEvent(new CustomEvent('ree-pak:release-preview-files'));
document.querySelectorAll('audio, video').forEach((element) => {
  element.pause();
  element.removeAttribute('src');
  element.load();
});
document.querySelectorAll('img, source').forEach((element) => {
  element.removeAttribute('src');
  element.removeAttribute('srcset');
});
"#;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
static MAIN_WINDOW_CLOSING: AtomicBool = AtomicBool::new(false);

fn panic_hook(info: &std::panic::PanicHookInfo) {
    #[cfg(target_os = "windows")]
    utility::message_box_error(&format!("panic occurred: {:#}", info));
    #[cfg(not(target_os = "windows"))]
    eprintln!("panic occurred: {:#}", info);
    std::process::exit(1);
}

// Clean temp files.
fn clean_temp_files() -> bool {
    let temp_dir = get_local_dir().join(TEMP_DIR_NAME);
    if !temp_dir.exists() {
        return true;
    }

    if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
        log::warn!("Failed to clean temp files: {}", e);
        return false;
    }

    log::info!("Temp files cleaned: {:?}", temp_dir);
    true
}

fn release_preview_file_references(window: &tauri::WebviewWindow) {
    if let Err(e) = window.eval(RELEASE_PREVIEW_REFERENCES_SCRIPT) {
        log::warn!("Failed to release preview file references: {}", e);
    }
}

fn close_main_window_after_temp_cleanup(window: tauri::WebviewWindow) {
    tauri::async_runtime::spawn(async move {
        for delay in [150, 300, 600] {
            tokio::time::sleep(Duration::from_millis(delay)).await;
            if clean_temp_files() {
                break;
            }
        }

        if let Err(e) = window.destroy() {
            log::warn!("Failed to close main window after temp cleanup: {}", e);
        }
    });
}

fn handle_main_window_close_requested(window: &tauri::WebviewWindow, api: &tauri::CloseRequestApi) {
    api.prevent_close();

    if MAIN_WINDOW_CLOSING.swap(true, Ordering::SeqCst) {
        return;
    }

    release_preview_file_references(window);
    close_main_window_after_temp_cleanup(window.clone());
}

fn register_main_window_close_cleanup(window: &tauri::WebviewWindow) {
    let main_window = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            handle_main_window_close_requested(&main_window, api);
        }
    });
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
    let _ = AudioService::initialize();
    let _ = PreviewService::initialize();
    let _ = ModelInsightService::initialize();

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
            disable_default_webview_context_menu(&main_window);
            register_main_window_close_cleanup(&main_window);
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
            command::pak_analyze_conflicts,
            command::pak_pack,
            command::pak_terminate_pack,
            command::file_table_load,
            command::file_table_push_paths,
            command::get_preview_file,
            command::audio_list_container,
            command::audio_extract_wems,
            command::audio_extract_wavs,
            command::audio_extract_wavs_with_progress,
            command::audio_terminate_extract,
            command::vgmstream_get_status,
            command::model_insight_load_mesh_assets,
            command::vgmstream_install_from_archive,
            command::export_texture_files,
            command::terminate_texture_export,
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

#[cfg(windows)]
fn disable_default_webview_context_menu(window: &tauri::WebviewWindow) {
    if let Err(error) = window.with_webview(|webview| unsafe {
        let result = webview
            .controller()
            .CoreWebView2()
            .and_then(|core| core.Settings())
            .and_then(|settings| settings.SetAreDefaultContextMenusEnabled(false));

        if let Err(error) = result {
            log::warn!("Failed to disable WebView2 default context menu: {error}");
        }
    }) {
        log::warn!("Failed to access WebView2 for context menu setup: {error}");
    }
}

#[cfg(not(windows))]
fn disable_default_webview_context_menu(_window: &tauri::WebviewWindow) {}
