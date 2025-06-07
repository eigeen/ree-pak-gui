use std::{fs::File, io::BufReader};

use anyhow::Context as _;
use ree_pak_core::filename::FileNameTable;
use tauri::ipc::Channel;

use crate::{
    channel::{ProgressChannel, WorkProgressEvent},
    pak::{
        ExtractOptions, PakId, PakInfo,
        tree::{FileTree, RenderTreeNode, RenderTreeOptions},
    },
    service::{
        filelist::{FileListInfo, FileListService},
        pak::PakService,
    },
    utility, warp_result_elapsed,
};

type PakServiceState = PakService<BufReader<File>>;

/// Clear all loaded Pak files.
#[tauri::command]
pub fn pak_clear_all(pak_service: tauri::State<PakServiceState>) -> Result<(), String> {
    pak_service.clear_all_paks();
    Ok(())
}

/// List all loaded Pak files.
#[tauri::command]
pub fn pak_list_all(pak_service: tauri::State<PakServiceState>) -> Result<Vec<PakInfo>, String> {
    Ok(pak_service.list_all_paks())
}

/// Open a Pak file.
#[tauri::command]
pub fn pak_open(pak_service: tauri::State<PakServiceState>, path: &str) -> Result<PakId, String> {
    pak_service.open_pak(path).map_err(|e| e.to_string())
}

/// Close a Pak file.
#[tauri::command]
pub fn pak_close(pak_service: tauri::State<PakServiceState>, id: PakId) -> Result<(), String> {
    pak_service.close_pak(id).map_err(|e| e.to_string())
}

/// Set the order of loaded Pak files.
#[tauri::command]
pub fn pak_order(pak_service: tauri::State<PakServiceState>, order: Vec<PakId>) -> Result<(), String> {
    pak_service.order_paks(&order).map_err(|e| e.to_string())
}

/// Get the information of a Pak file.
#[tauri::command]
pub fn pak_get_info(pak_service: tauri::State<PakServiceState>, id: PakId) -> Result<PakInfo, String> {
    pak_service.get_pak_info(id).map_err(|e| e.to_string())
}

/// (legacy) Read the file tree of current Pak group.
///
/// Should load file name list first.
#[tauri::command]
pub fn pak_read_file_tree(pak_service: tauri::State<PakServiceState>) -> Result<FileTree, String> {
    pak_service.read_file_tree().map_err(|e| e.to_string())
}

/// Read the file tree of current Pak group.
///
/// Structure optimized for frontend rendering.
///
/// Should load file name list first.
#[tauri::command]
pub fn pak_read_file_tree_optimized(
    pak_service: tauri::State<PakServiceState>,
    options: Option<RenderTreeOptions>,
) -> Result<RenderTreeNode, String> {
    warp_result_elapsed!(
        pak_service.read_file_tree_optimized(&options.unwrap_or_default()),
        "read_file_tree_optimized spent {} ms"
    )
}

/// Extract all loaded paks.
#[tauri::command]
pub fn pak_extract_all(
    pak_service: tauri::State<PakServiceState>,
    options: ExtractOptions,
    on_event: Channel<WorkProgressEvent>,
) -> Result<(), String> {
    if options.extract_all {
        println!("Extracting all entries...");
    } else {
        println!("Extracting {} entries...", options.extract_files.len());
    }

    let channel = ProgressChannel::new(on_event);
    pak_service
        .unpack_optional(&options, channel)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Terminate the current extraction process.
#[tauri::command]
pub fn pak_terminate_extraction(pak_service: tauri::State<PakServiceState>) -> Result<(), String> {
    pak_service.terminate_unpack();
    log::warn!("Extraction process terminated.");
    Ok(())
}

/// List all .list files in the file table directory.
#[tauri::command]
pub fn file_table_get_list(file_list_service: tauri::State<FileListService>) -> Result<Vec<FileListInfo>, String> {
    file_list_service.get_file_lists().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn file_table_load(pak_service: tauri::State<PakServiceState>, path: &str) -> Result<(), String> {
    warp_result_elapsed!(
        {
            let table = FileNameTable::from_list_file(path).map_err(|e| e.to_string())?;
            pak_service.set_file_name_table(table);
            Ok::<(), String>(())
        },
        "file_table_load spent {} ms"
    )
}

#[tauri::command]
pub fn get_exe_path() -> Result<String, String> {
    let path = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileInfo {
    version: &'static str,
    commit_time: &'static str,
    commit_hash: &'static str,
    platform: &'static str,
    arch: &'static str,
}

#[tauri::command]
pub fn get_compile_info() -> CompileInfo {
    CompileInfo {
        version: env!("CARGO_PKG_VERSION"),
        commit_time: env!("GIT_COMMIT_TIME_RFC3339"),
        commit_hash: env!("GIT_COMMIT_HASH"),
        platform: std::env::consts::OS,
        arch: std::env::consts::ARCH,
    }
}

/// Replace current binary with the given file.
///
/// Will apply after restart.
#[tauri::command]
pub fn perform_update(file_path: String) -> Result<(), String> {
    self_replace::self_replace(&file_path)
        .context("Failed to replace current binary")
        .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&file_path);

    Ok(())
}

#[tauri::command]
pub fn zip_extract_file(file_path: String, output_path: Option<String>) -> Result<(), String> {
    let output_path = output_path.unwrap_or_else(|| ".".to_string());
    utility::zip_extract_all(file_path, output_path).map_err(|e| e.to_string())?;

    Ok(())
}
