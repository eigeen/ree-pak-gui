use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::common::JsSafeHash;
use crate::error::Result;

use super::PakId;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTree {
    pub roots: Vec<FileTreeNode>,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub file_count: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTreeNode {
    pub info: NodeInfo,
    pub children: HashMap<SmolStr, FileTreeNode>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub is_dir: bool,
    pub relative_path: SmolStr,
    pub hash: Option<JsSafeHash>,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub is_compressed: bool,
    /// Belonging to which pak.
    /// If node is a directory, it will be None.
    pub belongs_to: Option<PakId>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderTreeOptions {
    sort_by_name: Option<bool>,
    sort_by_size: Option<bool>,
}

impl Default for RenderTreeOptions {
    fn default() -> Self {
        Self {
            sort_by_name: Some(true),
            sort_by_size: Some(false),
        }
    }
}

impl RenderTreeOptions {
    pub fn sort_by_name(&self) -> bool {
        self.sort_by_name.unwrap_or(true)
    }

    pub fn sort_by_size(&self) -> bool {
        self.sort_by_size.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderTreeNode {
    /// 是否是目录
    pub is_dir: bool,
    /// 节点名称，文件名或组合目录名
    pub name: SmolStr,
    /// 节点哈希值，如果是目录，则为None
    pub hash: Option<JsSafeHash>,
    /// 节点大小或目录所有子节点压缩后大小
    pub compressed_size: u64,
    /// 节点大小或目录所有子节点压缩前大小
    pub uncompressed_size: u64,
    /// 节点所属的 Pak
    pub belongs_to: Option<PakId>,
    /// 子节点
    pub children: Vec<RenderTreeNode>,
}

impl RenderTreeNode {
    pub fn try_from_file_tree(
        file_tree: FileTree,
        options: &RenderTreeOptions,
    ) -> Result<Vec<Self>> {
        let mut roots = file_tree
            .roots
            .iter()
            .map(convert_to_render_node)
            .collect::<Vec<_>>();

        if options.sort_by_name() {
            sort_by_name(&mut roots);
        } else if options.sort_by_size() {
            // TODO
        }

        for root in &mut roots {
            apply_dir_size(root);
        }

        Ok(roots)
    }
}

fn convert_to_render_node(node: &FileTreeNode) -> RenderTreeNode {
    let info = &node.info;

    RenderTreeNode {
        is_dir: info.is_dir,
        name: info.relative_path.clone(),
        hash: info.hash,
        compressed_size: info.compressed_size,
        uncompressed_size: info.uncompressed_size,
        belongs_to: info.belongs_to,
        children: node.children.values().map(convert_to_render_node).collect(),
    }
}

fn apply_dir_size(node: &mut RenderTreeNode) {
    if node.is_dir {
        let mut total_compressed_size = 0;
        let mut total_uncompressed_size = 0;

        for child in &mut node.children {
            apply_dir_size(child);
            total_compressed_size += child.compressed_size;
            total_uncompressed_size += child.uncompressed_size;
        }

        node.compressed_size = total_compressed_size;
        node.uncompressed_size = total_uncompressed_size;
    }
}

fn sort_by_name(nodes: &mut [RenderTreeNode]) {
    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });
    for node in nodes {
        sort_by_name(&mut node.children);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory(name: &str, children: Vec<FileTreeNode>) -> FileTreeNode {
        FileTreeNode {
            info: NodeInfo {
                is_dir: true,
                relative_path: SmolStr::new(name),
                ..NodeInfo::default()
            },
            children: children
                .into_iter()
                .map(|child| (child.info.relative_path.clone(), child))
                .collect(),
        }
    }

    fn file(name: &str, size: u64) -> FileTreeNode {
        FileTreeNode {
            info: NodeInfo {
                is_dir: false,
                relative_path: SmolStr::new(name),
                compressed_size: size,
                uncompressed_size: size,
                ..NodeInfo::default()
            },
            children: HashMap::new(),
        }
    }

    #[test]
    fn render_tree_keeps_real_directory_chain() {
        let tree = FileTree {
            roots: vec![directory(
                "A",
                vec![directory(
                    "B",
                    vec![directory("C", vec![file("x.tex", 12)])],
                )],
            )],
            ..FileTree::default()
        };

        let roots = RenderTreeNode::try_from_file_tree(tree, &RenderTreeOptions::default())
            .expect("tree should render");

        assert_eq!(roots[0].name, "A");
        assert_eq!(roots[0].children[0].name, "B");
        assert_eq!(roots[0].children[0].children[0].name, "C");
        assert_eq!(roots[0].children[0].children[0].children[0].name, "x.tex");
    }
}
