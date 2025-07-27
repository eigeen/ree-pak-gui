use anyhow::Context as _;
use ree_pak_core::filename::{FileNameTable, murmur3_hash};

use crate::{
    channel::{PackProgressChannel, PackProgressChannelInner, UnpackProgressChannel, UnpackProgressChannelInner},
    common::JsSafeHash,
    pak::{
        ExtractOptions, PakId, PakInfo,
        tree::{FileTree, RenderTreeNode, RenderTreeOptions},
    },
    service::{
        pak::{PakHeaderInfo, PakService},
        preview::PreviewService,
    },
    utility, warp_result_elapsed,
};

/// Clear all loaded Pak files.
#[tauri::command]
pub fn pak_clear_all() -> Result<(), String> {
    let pak_service = PakService::get();
    pak_service.clear_all_paks();
    Ok(())
}

/// List all loaded Pak files.
#[tauri::command]
pub fn pak_list_all() -> Result<Vec<PakInfo>, String> {
    let pak_service = PakService::get();
    Ok(pak_service.list_all_paks())
}

/// Open a Pak file.
#[tauri::command]
pub fn pak_open(path: &str) -> Result<PakId, String> {
    let pak_service = PakService::get();
    pak_service.open_pak(path).map_err(|e| e.to_string())
}

/// Close a Pak file.
#[tauri::command]
pub fn pak_close(id: PakId) -> Result<(), String> {
    let pak_service = PakService::get();
    pak_service.close_pak(id).map_err(|e| e.to_string())
}

/// Set the order of loaded Pak files.
#[tauri::command]
pub fn pak_order(order: Vec<PakId>) -> Result<(), String> {
    let pak_service = PakService::get();
    pak_service.order_paks(&order).map_err(|e| e.to_string())
}

/// Get the information of a Pak file.
#[tauri::command]
pub fn pak_get_info(id: PakId) -> Result<PakInfo, String> {
    let pak_service = PakService::get();
    pak_service.get_pak_info(id).map_err(|e| e.to_string())
}

/// (legacy) Read the file tree of current Pak group.
///
/// Should load file name list first.
#[tauri::command]
pub fn pak_read_file_tree() -> Result<FileTree, String> {
    let pak_service = PakService::get();
    pak_service.read_file_tree().map_err(|e| e.to_string())
}

/// Read the file tree of current Pak group.
///
/// Structure optimized for frontend rendering.
///
/// Should load file name list first.
#[tauri::command]
pub fn pak_read_file_tree_optimized(options: Option<RenderTreeOptions>) -> Result<RenderTreeNode, String> {
    let pak_service = PakService::get();
    warp_result_elapsed!(
        pak_service.read_file_tree_optimized(&options.unwrap_or_default()),
        "read_file_tree_optimized spent {} ms"
    )
}

/// Extract all loaded paks.
#[tauri::command]
pub fn pak_extract_all(options: ExtractOptions, on_event: UnpackProgressChannelInner) -> Result<(), String> {
    let pak_service = PakService::get();
    if options.extract_all {
        println!("Extracting all entries...");
    } else {
        println!("Extracting {} entries...", options.extract_files.len());
    }

    let channel = UnpackProgressChannel::new(on_event);
    pak_service
        .unpack_optional(&options, channel)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Terminate the current extraction process.
#[tauri::command]
pub fn pak_terminate_extraction() -> Result<(), String> {
    let pak_service = PakService::get();
    pak_service.terminate_work();
    log::warn!("Extraction process terminated.");
    Ok(())
}

#[tauri::command]
pub fn pak_get_header(pak_path: &str) -> Result<PakHeaderInfo, String> {
    PakService::get_header(pak_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pak_pack(sources: Vec<String>, output: String, on_event: PackProgressChannelInner) -> Result<(), String> {
    let pak_service = PakService::get();
    let channel = PackProgressChannel::new(on_event);
    pak_service.pack(&sources, &output, channel).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pak_terminate_pack() -> Result<(), String> {
    let pak_service = PakService::get();
    pak_service.terminate_work();
    log::warn!("Pack process terminated.");
    Ok(())
}

#[tauri::command]
pub fn file_table_load(path: &str) -> Result<(), String> {
    let pak_service = PakService::get();
    warp_result_elapsed!(
        {
            let table = FileNameTable::from_list_file(path).map_err(|e| e.to_string())?;
            pak_service.set_file_name_table(table);
            Ok::<(), String>(())
        },
        "file_table_load spent {} ms"
    )
}

/// Get preview file path.
///
/// Will return error if the file is not supported.
#[tauri::command]
pub async fn get_preview_file(hash: JsSafeHash) -> Result<String, String> {
    let preview_service = PreviewService::get();

    preview_service
        .get_preview_file(hash.hash_u64())
        .await
        .map_err(|e| e.to_string())
        .map(|p| p.to_string_lossy().to_string())
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

#[tauri::command]
pub fn murmur32(buffer: Vec<u8>) -> Result<u32, String> {
    murmur3_hash(&mut buffer.as_slice()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn murmur32_utf16(str: String) -> Result<u64, String> {
    use ree_pak_core::filename::FileNameExt;
    Ok(str.hash_mixed())
}
