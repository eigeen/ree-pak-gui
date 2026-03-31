use hashbrown::{HashMap, HashSet};
use ree_pak_core::filename::FileNameTable;

use super::{FileTreeStats, Pak, PakId, PakInfo, insert_tree_entry, tree::FileTree};

use crate::error::{Error, Result};

/// Manages a group of paks.
pub struct PakGroup {
    paks: Vec<Pak>,
    file_name_table: Option<FileNameTable>,
}

impl PakGroup {
    pub fn paks(&self) -> &[Pak] {
        &self.paks
    }

    pub fn paks_mut(&mut self) -> &mut [Pak] {
        &mut self.paks
    }

    pub fn file_name_table(&self) -> Option<&FileNameTable> {
        self.file_name_table.as_ref()
    }

    pub fn file_name_table_mut(&mut self) -> Option<&mut FileNameTable> {
        self.file_name_table.as_mut()
    }

    pub fn pak_infos(&self) -> Vec<PakInfo> {
        self.paks
            .iter()
            .map(|pak| PakInfo {
                id: pak.id,
                path: pak.path.clone(),
            })
            .collect()
    }

    pub fn total_files(&self) -> u64 {
        self.paks
            .iter()
            .map(|pak| pak.pakfile.metadata().entries().len() as u64)
            .sum()
    }

    pub fn add_pak(&mut self, pak: Pak) {
        if let Some(prev_pak) = self.get_pak_by_path(&pak.path) {
            // remove previous pak with same path
            let id: PakId = prev_pak.id;
            self.remove_pak(&id);
        }
        self.paks.push(pak);
    }

    pub fn get_pak(&self, id: &PakId) -> Option<&Pak> {
        self.paks.iter().find(|pak| pak.id == *id)
    }

    pub fn get_pak_by_path(&self, path: &str) -> Option<&Pak> {
        self.paks.iter().find(|pak| pak.path == path)
    }

    pub fn remove_pak(&mut self, id: &PakId) -> Option<Pak> {
        self.paks
            .iter()
            .position(|pak| pak.id == *id)
            .map(|i| self.paks.remove(i))
    }

    pub fn remove_all_paks(&mut self) {
        self.paks.clear();
    }

    pub fn set_file_name_table(&mut self, file_name_table: FileNameTable) {
        self.file_name_table = Some(file_name_table);
    }

    /// 联合解析所有已加载的 Pak 文件树
    pub fn render_tree_combined(&self) -> Result<FileTree> {
        let Some(file_name_table) = self.file_name_table.as_ref() else {
            return Err(Error::MissingFileList);
        };

        if self.paks.is_empty() {
            return Ok(FileTree::default());
        }

        let mut root_children = HashMap::new();
        let mut stats = FileTreeStats::default();
        let mut seen_hashes = HashSet::new();

        for pak in self.paks.iter().rev() {
            for entry in pak.pakfile.metadata().entries() {
                if !seen_hashes.insert(entry.hash()) {
                    continue;
                }
                insert_tree_entry(
                    &mut root_children,
                    &mut stats,
                    pak.id,
                    file_name_table,
                    entry,
                );
            }
        }

        Ok(FileTree {
            roots: root_children.into_values().collect(),
            uncompressed_size: stats.uncompressed_size,
            compressed_size: stats.compressed_size,
            file_count: stats.file_count,
        })
    }
}

impl PakGroup {
    pub fn new() -> Self {
        Self {
            paks: Vec::new(),
            file_name_table: None,
        }
    }
}
