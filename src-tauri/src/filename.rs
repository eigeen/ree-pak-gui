use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListInfo {
    pub name: String,
    pub abs_path: String,
}

impl FileListInfo {
    pub fn walk_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<FileListInfo>> {
        let mut files: Vec<FileListInfo> = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() || path.extension().unwrap_or_default() != "list" {
                continue;
            }
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let abs_path = path.to_string_lossy().to_string();
            files.push(FileListInfo { name, abs_path });
        }

        Ok(files)
    }
}
