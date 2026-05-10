use ree_pak_core::filename::FileNameTable;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Instant};
use tauri::AppHandle;

use crate::{
    channel::{
        AudioExportProgressChannel, AudioExportProgressChannelInner, FileTreeProgressChannel,
        FileTreeProgressChannelInner, PackProgressChannel, PackProgressChannelInner,
        TextureExportProgressChannel, TextureExportProgressChannelInner, UnpackProgressChannel,
        UnpackProgressChannelInner,
    },
    common::JsSafeHash,
    external_tools::{self, VgmstreamStatus},
    pak::{
        ExtractFileInfo, ExtractOptions, PakId, PakInfo,
        tree::{FileTree, RenderTreeOptions},
    },
    service::{
        audio::{AudioContainerInfo, AudioExtractBatchOptions, AudioService, AudioSourceRef},
        model_insight::{
            ModelInsightLoadMeshAssetsOptions, ModelInsightLoadTexturePreviewsOptions,
            ModelInsightMeshAssets, ModelInsightService, ModelInsightTexturePreview,
        },
        pak::{PackConflictInfo, PakHeaderInfo, PakService},
        preview::{PreviewService, TextureExportFormat},
        update::{AppUpdateInfo, AppUpdateProgressChannel},
    },
    utility, warp_result_elapsed,
};

fn log_sync_command<T>(
    name: &str,
    detail: Option<String>,
    command: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    if let Some(detail) = detail.as_deref() {
        log::info!("[command:{name}] start {detail}");
    } else {
        log::info!("[command:{name}] start");
    }

    let start = Instant::now();
    let result = command();
    let elapsed_ms = start.elapsed().as_millis();

    match &result {
        Ok(_) => log::info!("[command:{name}] done elapsed={elapsed_ms} ms"),
        Err(error) => {
            log::error!("[command:{name}] failed elapsed={elapsed_ms} ms error={error}")
        }
    }

    result
}

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
    let loaded_pak_count = pak_service.list_all_paks().len();
    log_sync_command(
        "pak_read_file_tree",
        Some(format!("loaded_paks={loaded_pak_count}")),
        || pak_service.read_file_tree().map_err(|e| e.to_string()),
    )
}

/// Read the file tree of current Pak group.
///
/// Structure optimized for frontend rendering.
///
/// Should load file name list first.
#[tauri::command]
pub async fn pak_read_file_tree_optimized(
    options: Option<RenderTreeOptions>,
    on_event: FileTreeProgressChannelInner,
) -> Result<(), String> {
    let pak_service = PakService::get();
    let progress = FileTreeProgressChannel::new(on_event);
    warp_result_elapsed!(
        pak_service
            .read_file_tree_optimized_async(options.unwrap_or_default(), progress)
            .await,
        "read_file_tree_optimized spent {} ms"
    )
}

/// Extract all loaded paks.
#[tauri::command]
pub async fn pak_extract_all(
    options: ExtractOptions,
    on_event: UnpackProgressChannelInner,
) -> Result<(), String> {
    let pak_service = PakService::get();
    if options.extract_all {
        println!("Extracting all entries...");
    } else {
        println!("Extracting {} entries...", options.extract_files.len());
    }

    let channel = UnpackProgressChannel::new(on_event);
    pak_service
        .unpack_optional(&options, channel)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Terminate the current extraction process.
#[tauri::command]
pub fn pak_terminate_extraction() -> Result<(), String> {
    let pak_service = PakService::get();
    pak_service.terminate_unpack();
    log::warn!("Extraction process terminated.");
    Ok(())
}

#[tauri::command]
pub fn pak_get_header(pak_path: &str) -> Result<PakHeaderInfo, String> {
    log_sync_command("pak_get_header", Some(format!("path={pak_path}")), || {
        PakService::get_header(pak_path).map_err(|e| e.to_string())
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackOptions {
    pub sources: Vec<String>,
    pub output: String,
    pub allow_file_name_as_path_hash: bool,
    #[serde(default)]
    pub conflict_resolutions: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAnalyzeOptions {
    pub sources: Vec<String>,
    pub allow_file_name_as_path_hash: bool,
}

#[tauri::command]
pub fn pak_analyze_conflicts(options: PackAnalyzeOptions) -> Result<Vec<PackConflictInfo>, String> {
    let pak_service = PakService::get();
    let source_count = options.sources.len();
    log_sync_command(
        "pak_analyze_conflicts",
        Some(format!("sources={source_count} phase=analyze")),
        || {
            pak_service
                .analyze_conflicts(&options)
                .map_err(|e| e.to_string())
        },
    )
}

#[tauri::command]
pub fn pak_pack(options: PackOptions, on_event: PackProgressChannelInner) -> Result<(), String> {
    let pak_service = PakService::get();
    let channel = PackProgressChannel::new(on_event);
    let source_count = options.sources.len();
    let output = options.output.clone();
    log_sync_command(
        "pak_pack",
        Some(format!(
            "sources={source_count} output={output} phase=dispatch"
        )),
        || {
            pak_service
                .pack(&options, channel)
                .map_err(|e| e.to_string())
        },
    )
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
    log_sync_command("file_table_load", Some(format!("path={path}")), || {
        let table = FileNameTable::from_list_file(path).map_err(|e| e.to_string())?;
        pak_service.set_file_name_table(table);
        Ok::<(), String>(())
    })
}

#[tauri::command]
pub fn file_table_push_paths(file_path_list: Vec<String>) -> Result<(), String> {
    let pak_service = PakService::get();
    pak_service.push_file_paths(file_path_list);
    Ok(())
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
pub async fn audio_list_container(source: AudioSourceRef) -> Result<AudioContainerInfo, String> {
    let audio_service = AudioService::get();

    tokio::task::spawn_blocking(move || audio_service.list_container(source))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn audio_extract_wems(options: AudioExtractBatchOptions) -> Result<Vec<String>, String> {
    let audio_service = AudioService::get();

    tokio::task::spawn_blocking(move || audio_service.extract_wems(options))
        .await
        .map_err(|error| error.to_string())?
        .map(|paths| {
            paths
                .into_iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect()
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn audio_extract_wavs(options: AudioExtractBatchOptions) -> Result<Vec<String>, String> {
    let audio_service = AudioService::get();

    tokio::task::spawn_blocking(move || audio_service.extract_wavs(options))
        .await
        .map_err(|error| error.to_string())?
        .map(|paths| {
            paths
                .into_iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect()
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn audio_extract_wavs_with_progress(
    options: AudioExtractBatchOptions,
    on_event: AudioExportProgressChannelInner,
) -> Result<Vec<String>, String> {
    let audio_service = AudioService::get();
    let progress = AudioExportProgressChannel::new(on_event);

    tokio::task::spawn_blocking(move || audio_service.extract_wavs_with_progress(options, progress))
        .await
        .map_err(|error| error.to_string())?
        .map(|paths| {
            paths
                .into_iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect()
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn audio_terminate_extract() -> Result<(), String> {
    AudioService::get().terminate_extract();
    log::warn!("Audio export process terminated.");
    Ok(())
}

#[tauri::command]
pub fn vgmstream_get_status() -> VgmstreamStatus {
    external_tools::vgmstream_status()
}

#[tauri::command]
pub async fn model_insight_load_mesh_assets(
    options: ModelInsightLoadMeshAssetsOptions,
) -> Result<ModelInsightMeshAssets, String> {
    let detail = Some(format!("entry_path={}", options.entry_path));
    tokio::task::spawn_blocking(move || {
        log_sync_command("model_insight_load_mesh_assets", detail, || {
            ModelInsightService::get()
                .load_mesh_assets(options)
                .map_err(|error| error.to_string())
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn model_insight_load_texture_previews(
    options: ModelInsightLoadTexturePreviewsOptions,
) -> Result<Vec<ModelInsightTexturePreview>, String> {
    let detail = Some(format!(
        "base_entry_path={}, textures={}",
        options.base_entry_path,
        options.texture_paths.len()
    ));
    tokio::task::spawn_blocking(move || {
        log_sync_command("model_insight_load_texture_previews", detail, || {
            ModelInsightService::get()
                .load_texture_previews(options)
                .map_err(|error| error.to_string())
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn vgmstream_install_from_archive(archive_path: String) -> Result<VgmstreamStatus, String> {
    log_sync_command(
        "vgmstream_install_from_archive",
        Some(format!("archive_path={archive_path}")),
        || {
            external_tools::install_vgmstream_from_archive(archive_path.clone())
                .map_err(|error| error.to_string())
        },
    )
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureExportOptions {
    pub output_path: String,
    pub format: String,
    pub files: Vec<ExtractFileInfo>,
}

#[tauri::command]
pub async fn export_texture_files(
    options: TextureExportOptions,
    on_event: TextureExportProgressChannelInner,
) -> Result<usize, String> {
    let preview_service = PreviewService::get();
    let format = match options.format.as_str() {
        "dds" => TextureExportFormat::Dds,
        "png" => TextureExportFormat::Png,
        other => return Err(format!("Unsupported texture export format: {other}")),
    };
    let progress = TextureExportProgressChannel::new(on_event);

    preview_service
        .export_texture_files(format, &options.output_path, &options.files, progress)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn terminate_texture_export() -> Result<(), String> {
    let preview_service = PreviewService::get();
    preview_service.terminate_export();
    log::warn!("Texture export process terminated.");
    Ok(())
}

#[tauri::command]
pub fn get_exe_path() -> Result<String, String> {
    let path = std::env::current_exe().map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[derive(Serialize)]
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

#[tauri::command]
pub fn app_check_update() -> Result<Option<AppUpdateInfo>, String> {
    log_sync_command(
        "app_check_update",
        Some("source=github_releases".to_string()),
        || crate::service::update::check_for_update().map_err(|e| e.to_string()),
    )
}

#[tauri::command]
pub async fn app_install_update(
    app: AppHandle,
    on_event: AppUpdateProgressChannel,
) -> Result<(), String> {
    crate::service::update::install_update(app, on_event).await
}

#[tauri::command]
pub fn zip_extract_file(file_path: String, output_path: Option<String>) -> Result<(), String> {
    let output_path = output_path.unwrap_or_else(|| ".".to_string());
    log_sync_command(
        "zip_extract_file",
        Some(format!("file_path={file_path} output_path={output_path}")),
        || {
            utility::zip_extract_all(file_path.clone(), output_path.clone())
                .map(|_| ())
                .map_err(|e| e.to_string())
        },
    )
}

#[tauri::command]
pub fn murmur32(buffer: Vec<u8>) -> Result<u32, String> {
    ree_pak_core::utf16_hash::murmur3_hash(&mut buffer.as_slice()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn murmur32_utf16(str: String) -> Result<JsSafeHash, String> {
    use ree_pak_core::utf16_hash::Utf16HashExt;
    Ok(JsSafeHash::from_u64(str.hash_mixed()))
}
