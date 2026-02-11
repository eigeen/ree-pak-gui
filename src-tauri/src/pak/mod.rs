use std::path::PathBuf;

use ree_pak_core::{filename::FileNameTable, pak::CompressionType, pakfile::PakFile};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tree::{FileTree, FileTreeNode, NodeInfo};

use crate::common::{JsSafeHash, UniqueId};

pub mod group;
pub mod tree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PakId(UniqueId);

impl From<UniqueId> for PakId {
    fn from(value: UniqueId) -> Self {
        PakId(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PakInfo {
    pub id: PakId,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractOptions {
    pub output_path: String,
    pub r#override: bool,
    pub extract_all: bool,
    pub extract_files: Vec<ExtractFileInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractFileInfo {
    pub hash: JsSafeHash,
    pub belongs_to: PakId,
}

pub struct Pak {
    pub(crate) id: PakId,
    pub(crate) path: String,
    pub(crate) pakfile: PakFile,
}

impl Pak {
    pub fn new(path: &str, pakfile: PakFile) -> Pak {
        Pak {
            id: UniqueId::create().into(),
            path: path.to_string(),
            pakfile,
        }
    }

    pub fn create_tree(&self, name_table: &FileNameTable) -> FileTree {
        let mut root = FileTreeNode {
            info: NodeInfo {
                is_dir: true,
                relative_path: "/".to_string(),
                hash: None,
                uncompressed_size: 0,
                compressed_size: 0,
                is_compressed: false,
                belongs_to: None,
            },
            children: FxHashMap::default(),
        };

        let mut total_uncompressed_size = 0_u64;
        let mut total_compressed_size = 0_u64;
        let mut total_file_count = 0_u64;

        self.pakfile.archive().entries().iter().for_each(|entry| {
            let file_relative_path: PathBuf = name_table
                .get_file_name(entry.hash())
                .map(|fname| fname.to_string().unwrap())
                .unwrap_or_else(|| format!("_Unknown/{:08X}", entry.hash()))
                .into();
            let components: Vec<&str> = file_relative_path
                .components()
                .map(|c| c.as_os_str().to_str().unwrap())
                .collect::<Vec<_>>();
            let mut current_node = &mut root;

            for (i, component) in components.iter().enumerate() {
                let is_dir = i < components.len() - 1;
                // create or get the child node
                let child_node = current_node
                    .children
                    .entry(component.to_string())
                    .or_insert_with(|| FileTreeNode {
                        info: NodeInfo {
                            is_dir,
                            relative_path: component.to_string(),
                            hash: None,
                            uncompressed_size: 0,
                            compressed_size: 0,
                            is_compressed: false,
                            belongs_to: if is_dir { None } else { Some(self.id) },
                        },
                        children: FxHashMap::default(),
                    });
                if !is_dir {
                    child_node.info.uncompressed_size = entry.uncompressed_size();
                    child_node.info.compressed_size = entry.compressed_size();
                    child_node.info.is_compressed = entry.compression_type() != CompressionType::NONE;
                    child_node.info.hash = Some(JsSafeHash::from_u64(entry.hash()));
                    total_uncompressed_size += entry.uncompressed_size();
                    total_compressed_size += entry.compressed_size();
                    total_file_count += 1;
                }
                // move to the child node
                current_node = child_node;
            }
        });

        FileTree {
            root,
            uncompressed_size: total_uncompressed_size,
            compressed_size: total_compressed_size,
            file_count: total_file_count,
        }
    }
}
