use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, LazyLock, Mutex},
};

use ree_pak_core::filename::FileNameTable;

use super::{tree::FileTree, ExtractOptions, Pak, PakId, PakInfo};

use crate::{
    error::{Error, Result},
    pak::unpack,
};

type BufReaderFile = BufReader<File>;
type PakBufReaderFile = Pak<BufReaderFile>;

static PAK_GROUP: LazyLock<Arc<Mutex<PakGroup<BufReader<File>>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(PakGroup::new())));

type SharedPakGroup = Arc<Mutex<PakGroup<BufReader<File>>>>;

/// Manages a group of paks.
pub struct PakGroup<R> {
    paks: Vec<Pak<R>>,
    file_name_table: Option<FileNameTable>,
}

impl PakGroup<BufReader<File>> {
    pub fn instance() -> SharedPakGroup {
        PAK_GROUP.clone()
    }

    pub fn new() -> Self {
        Self {
            paks: Vec::new(),
            file_name_table: None,
        }
    }

    #[inline]
    pub fn file_name_table(&self) -> Option<&FileNameTable> {
        self.file_name_table.as_ref()
    }

    #[inline]
    pub fn paks(&self) -> &[PakBufReaderFile] {
        &self.paks
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

    pub fn add_pak(&mut self, pak: PakBufReaderFile) {
        if let Some(prev_pak) = self.get_pak_by_path(&pak.path) {
            // remove previous pak with same path
            let id: PakId = prev_pak.id;
            self.remove_pak(&id);
        }
        self.paks.push(pak);
    }

    pub fn get_pak(&self, id: &PakId) -> Option<&PakBufReaderFile> {
        self.paks.iter().find(|pak| pak.id == *id)
    }

    pub fn get_pak_mut(&mut self, id: &PakId) -> Option<&mut PakBufReaderFile> {
        self.paks.iter_mut().find(|pak| pak.id == *id)
    }

    pub fn get_pak_by_path(&self, path: &str) -> Option<&PakBufReaderFile> {
        self.paks.iter().find(|pak| pak.path == path)
    }

    pub fn remove_pak(&mut self, id: &PakId) -> Option<PakBufReaderFile> {
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

    pub fn unpack_optional(&mut self, options: &ExtractOptions) -> Result<()> {
        if self.paks.is_empty() {
            return Err(Error::NoPaksLoaded);
        }
        if self.file_name_table.is_none() {
            return Err(Error::MissingFileList);
        }

        for pak in self.paks.iter_mut() {
            let file_name_table = self.file_name_table.as_ref().unwrap();
            if let Err(e) = unpack::unpack_parallel_error_continue(pak, file_name_table, options) {
                eprintln!("Error unpacking pak: {}", e);
            }
        }

        Ok(())
    }
}
