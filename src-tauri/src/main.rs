// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod command;
mod common;
mod error;
mod filename;
mod macros;
mod pak;
mod usecase;
mod utility;

fn panic_hook(info: &std::panic::PanicHookInfo) {
    #[cfg(target_os = "windows")]
    utility::message_box_error(&format!("panic occurred: {:#}", info));
    #[cfg(not(target_os = "windows"))]
    println!("panic occurred: {:#}", info);
}

fn main() {
    std::panic::set_hook(Box::new(panic_hook));

    tauri::Builder::default()
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
            command::pak_peek_extract_progress,
            command::file_table_get_list,
            command::file_table_load,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
