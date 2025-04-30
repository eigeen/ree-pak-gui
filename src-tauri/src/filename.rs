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
            let full_ext = path
                .iter()
                .next_back()
                .unwrap_or_default()
                .to_string_lossy()
                .split('.')
                .next_back()
                .unwrap_or_default()
                .to_lowercase();
            if !path.is_file() || [".list", ".list.zst"].contains(&full_ext.as_str()) {
                continue;
            }
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let abs_path = path.to_string_lossy().to_string();
            files.push(FileListInfo { name, abs_path });
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walk_dir() {
        let files = FileListInfo::walk_dir("../../assets/filelist").unwrap();
        eprintln!("files: {:#?}", files);
    }
}
