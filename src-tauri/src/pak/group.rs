use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
};

use ree_pak_core::filename::FileNameTable;

use super::{Pak, PakId, PakInfo, tree::FileTree};

use crate::error::{Error, Result};

type PakBufReaderFile = Pak<BufReader<File>>;

/// Manages a group of paks.
pub struct PakGroup<R> {
    paks: Vec<Pak<R>>,
    file_name_table: Option<FileNameTable>,
}

impl<R> PakGroup<R>
where
    R: Send + Sync + BufRead + Seek,
{
    pub fn file_name_table(&self) -> Option<&FileNameTable> {
        self.file_name_table.as_ref()
    }

    pub fn paks(&self) -> &[Pak<R>] {
        &self.paks
    }

    pub fn paks_mut(&mut self) -> &mut [Pak<R>] {
        &mut self.paks
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
        self.paks.iter().map(|pak| pak.archive.entries().len() as u64).sum()
    }

    pub fn add_pak(&mut self, pak: Pak<R>) {
        if let Some(prev_pak) = self.get_pak_by_path(&pak.path) {
            // remove previous pak with same path
            let id: PakId = prev_pak.id;
            self.remove_pak(&id);
        }
        self.paks.push(pak);
    }

    pub fn get_pak(&self, id: &PakId) -> Option<&Pak<R>> {
        self.paks.iter().find(|pak| pak.id == *id)
    }

    pub fn get_pak_mut(&mut self, id: &PakId) -> Option<&mut Pak<R>> {
        self.paks.iter_mut().find(|pak| pak.id == *id)
    }

    pub fn get_pak_by_path(&self, path: &str) -> Option<&Pak<R>> {
        self.paks.iter().find(|pak| pak.path == path)
    }

    pub fn remove_pak(&mut self, id: &PakId) -> Option<Pak<R>> {
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
        if self.file_name_table.is_none() {
            return Err(Error::MissingFileList);
        }

        if self.paks.is_empty() {
            Ok(FileTree::default())
        // } else if self.paks.len() == 1 {
        //     Ok(self.paks[0].create_tree(self.file_name_table.as_ref().unwrap()))
        } else {
            // render combined tree
            let file_name_table = self.file_name_table.as_ref().unwrap();
            let mut tree = self.paks[0].create_tree(file_name_table);
            for pak in self.paks.iter().skip(1) {
                let sub_tree = pak.create_tree(file_name_table);
                tree = tree.combine(sub_tree);
            }

            Ok(tree)
        }
    }
}

impl PakGroup<BufReader<File>> {
    pub fn new() -> Self {
        Self {
            paks: Vec::new(),
            file_name_table: None,
        }
    }
}
