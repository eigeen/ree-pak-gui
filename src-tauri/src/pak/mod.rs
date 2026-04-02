use std::sync::Arc;

use hashbrown::HashMap;
use ree_pak_core::{
    filename::FileNameTable,
    pak::{CompressionType, PakEntry},
    pakfile::PakFile,
};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tree::{FileTreeNode, NodeInfo};

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
    pub mode: ExtractMode,
    pub extract_all: bool,
    pub extract_files: Vec<ExtractFileInfo>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExtractMode {
    RelativePath,
    AbsolutePath,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractFileInfo {
    pub hash: JsSafeHash,
    pub belongs_to: PakId,
    pub relative_root: Option<String>,
}

pub struct Pak {
    pub(crate) id: PakId,
    pub(crate) path: String,
    pub(crate) pakfile: Arc<PakFile>,
}

impl Pak {
    pub fn new(path: &str, pakfile: PakFile) -> Pak {
        Pak {
            id: UniqueId::create().into(),
            path: path.to_string(),
            pakfile: Arc::new(pakfile),
        }
    }
}

#[derive(Default)]
pub(super) struct FileTreeStats {
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub file_count: u64,
}

pub(super) fn insert_tree_entry(
    root_children: &mut HashMap<SmolStr, FileTreeNode>,
    stats: &mut FileTreeStats,
    pak_id: PakId,
    name_table: &FileNameTable,
    entry: &PakEntry,
) {
    let file_relative_path = name_table
        .get_file_name(entry.hash())
        .map(|fname| fname.to_string().unwrap())
        .unwrap_or_else(|| format!("_Unknown/{:08X}", entry.hash()))
        .replace('\\', "/");
    let mut current_node = root_children;
    let mut components = file_relative_path
        .split('/')
        .filter(|component| !component.is_empty())
        .peekable();

    while let Some(component) = components.next() {
        let is_dir = components.peek().is_some();
        let component_name = SmolStr::new(component);
        let child_node = current_node
            .entry(component_name.clone())
            .or_insert_with(|| FileTreeNode {
                info: NodeInfo {
                    is_dir,
                    relative_path: component_name,
                    hash: None,
                    uncompressed_size: 0,
                    compressed_size: 0,
                    is_compressed: false,
                    belongs_to: if is_dir { None } else { Some(pak_id) },
                },
                children: HashMap::new(),
            });
        if !is_dir {
            child_node.info.uncompressed_size = entry.uncompressed_size();
            child_node.info.compressed_size = entry.compressed_size();
            child_node.info.is_compressed = entry.compression_type() != CompressionType::None;
            child_node.info.hash = Some(JsSafeHash::from_u64(entry.hash()));
            stats.uncompressed_size += entry.uncompressed_size();
            stats.compressed_size += entry.compressed_size();
            stats.file_count += 1;
        }
        current_node = &mut child_node.children;
    }
}
