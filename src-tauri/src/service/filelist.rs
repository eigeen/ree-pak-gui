use std::{env, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub struct FileListService {}

impl FileListService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_file_lists(&self) -> Result<Vec<FileListInfo>> {
        // try work path
        let mut work_path = env::current_dir().unwrap_or(".".into());
        work_path.push("assets");
        work_path.push("filelist");
        let result = FileListInfo::walk_dir(&work_path);
        if let Ok(result) = result {
            if !result.is_empty() {
                return Ok(result);
            }
        }

        // try exe path
        let mut exe_path = env::current_exe().unwrap_or(".".into());
        exe_path.push("assets");
        exe_path.push("filelist");
        let result = FileListInfo::walk_dir(&exe_path);
        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(Error::FileListNotFound(format!(
                "error: {}, path: {} + {}",
                e,
                work_path.display(),
                exe_path.display()
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListInfo {
    pub name: String,
    pub abs_path: String,
}

impl FileListInfo {
    pub fn walk_dir(path: impl AsRef<Path>) -> anyhow::Result<Vec<FileListInfo>> {
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
