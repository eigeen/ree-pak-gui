use crate::{
    filename::FileListInfo,
    pak::{
        tree::{FileTree, RenderTreeNode, RenderTreeOptions},
        ExtractOptions, ExtractProgress, PakId, PakInfo,
    },
    usecase, warp_result_elapsed,
};

/// Clear all loaded Pak files.
#[tauri::command]
pub fn pak_clear_all() -> Result<(), String> {
    usecase::pak_clear_all();
    Ok(())
}

/// List all loaded Pak files.
#[tauri::command]
pub fn pak_list_all() -> Result<Vec<PakInfo>, String> {
    Ok(usecase::pak_list_all())
}

/// Open a Pak file.
#[tauri::command]
pub fn pak_open(path: &str) -> Result<PakId, String> {
    usecase::pak_open(path).map_err(|e| e.to_string())
}

/// Close a Pak file.
#[tauri::command]
pub fn pak_close(id: PakId) -> Result<(), String> {
    usecase::pak_close(id).map_err(|e| e.to_string())
}

/// Get the information of a Pak file.
#[tauri::command]
pub fn pak_get_info(id: PakId) -> Result<PakInfo, String> {
    usecase::pak_get_info(id).map_err(|e| e.to_string())
}

/// (legacy) Read the file tree of current Pak group.
///
/// Should load file name list first.
#[tauri::command]
pub fn pak_read_file_tree() -> Result<FileTree, String> {
    usecase::pak_read_file_tree().map_err(|e| e.to_string())
}

/// Read the file tree of current Pak group.
///
/// Structure optimized for frontend rendering.
///
/// Should load file name list first.
#[tauri::command]
pub fn pak_read_file_tree_optimized(options: Option<RenderTreeOptions>) -> Result<RenderTreeNode, String> {
    warp_result_elapsed!(
        usecase::pak_read_file_tree_optimized(&options.unwrap_or_default()),
        "read_file_tree_optimized spent {} ms"
    )
}

/// Extract all loaded paks.
#[tauri::command]
pub fn pak_extract_all(options: ExtractOptions) -> Result<(), String> {
    if options.extract_all {
        println!("Extracting all entries...");
    } else {
        println!("Extracting {} entries...", options.extract_files.len());
    }

    warp_result_elapsed!(usecase::pak_extract_all(&options), "pak_extract_all spent {} ms")
}

/// Get the current extract progress.
#[tauri::command]
pub fn pak_peek_extract_progress(id: PakId) -> Result<ExtractProgress, String> {
    todo!()
}

/// List all .list files in the file table directory.
#[tauri::command]
pub fn file_table_get_list() -> Result<Vec<FileListInfo>, String> {
    usecase::get_file_lists().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn file_table_load(path: &str) -> Result<(), String> {
    warp_result_elapsed!(usecase::load_file_list(path), "file_table_load spent {} ms")
}
