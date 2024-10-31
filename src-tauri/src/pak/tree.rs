use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::common::JsSafeHash;
use crate::error::Result;

use super::PakId;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTree {
    pub root: FileTreeNode,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    pub file_count: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTreeNode {
    pub info: NodeInfo,
    pub children: FxHashMap<String, FileTreeNode>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub is_dir: bool,
    pub relative_path: String,
    pub hash: Option<JsSafeHash>,
    pub uncompressed_size: u64,
    pub compressed_size: u64,
    /// Belonging to which pak.
    /// If node is a directory, it will be None.
    pub belonging_to: Option<PakId>,
}

impl FileTree {
    /// Combine two file trees and return a new one.
    ///
    /// The same file will override by the new one.
    pub fn combine(self, other: FileTree) -> FileTree {
        let mut combined_root = self.root;

        // 合并两个树的根节点
        combined_root = Self::combine_nodes(combined_root, other.root);

        // TODO: 重新计算所有目录大小

        // 目录大小由其他函数独立计算
        let combined_uncompressed_size = 0;
        let combined_compressed_size = 0;
        let combined_file_count = 0;

        FileTree {
            root: combined_root,
            uncompressed_size: combined_uncompressed_size,
            compressed_size: combined_compressed_size,
            file_count: combined_file_count,
        }
    }

    fn combine_nodes(node1: FileTreeNode, node2: FileTreeNode) -> FileTreeNode {
        let mut combined_node = node1;

        // 目录，直接合并子节点
        if node2.info.is_dir {
            for (ref key, child_node2) in node2.children {
                // 如果key已经存在，则合并，否则直接插入
                let child_node1 = combined_node.children.entry(key.clone()).or_insert_with(|| {
                    // 如果不存在，直接克隆一个新的节点
                    FileTreeNode {
                        info: NodeInfo {
                            is_dir: true, // 新节点是目录
                            relative_path: key.clone(),
                            hash: None,
                            uncompressed_size: 0,
                            compressed_size: 0,
                            belonging_to: None,
                        },
                        children: FxHashMap::default(),
                    }
                });

                // 合并子节点
                *child_node1 = Self::combine_nodes(child_node1.clone(), child_node2);
            }
        } else {
            // 非目录覆盖
            combined_node.info = node2.info;
        }

        combined_node
    }

    /// 计算并更新所有父节点的大小
    fn update_dir_node_size(root: &mut FileTreeNode) {
        todo!();
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderTreeOptions {
    merge_directories: Option<bool>,
    sort_by_name: Option<bool>,
    sort_by_size: Option<bool>,
}

impl Default for RenderTreeOptions {
    fn default() -> Self {
        Self {
            merge_directories: Some(true),
            sort_by_name: Some(true),
            sort_by_size: Some(false),
        }
    }
}

impl RenderTreeOptions {
    pub fn merge_directories(&self) -> bool {
        self.merge_directories.unwrap_or(true)
    }

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
    pub name: String,
    /// 节点哈希值，如果是目录，则为None
    pub hash: Option<JsSafeHash>,
    /// 节点大小或目录所有子节点压缩后大小
    pub compressed_size: u64,
    /// 节点大小或目录所有子节点压缩前大小
    pub uncompressed_size: u64,
    /// 节点所属的 Pak
    pub belonging_to: Option<PakId>,
    /// 子节点
    pub children: Vec<RenderTreeNode>,
}

impl RenderTreeNode {
    pub fn try_from_file_tree(file_tree: FileTree, options: &RenderTreeOptions) -> Result<Self> {
        let mut root = convert_to_render_node(&file_tree.root);
        // 合并独立嵌套目录
        if options.merge_directories() {
            let mut _root = [root];
            merge_nested_dirs(&mut _root);
            root = _root.into_iter().next().unwrap();
        }
        // 排序
        if options.sort_by_name() {
            sort_by_name(&mut root.children);
        } else if options.sort_by_size() {
            // TODO
        }

        // 计算目录大小
        apply_dir_size(&mut root);

        Ok(root)
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
        belonging_to: info.belonging_to,
        children: node.children.values().map(convert_to_render_node).collect(),
    }
}

/// 计算目录包含的所有文件大小
fn apply_dir_size(node: &mut RenderTreeNode) {
    // 如果是目录，先初始化大小
    if node.is_dir {
        // 初始化压缩前后大小
        let mut total_compressed_size = 0;
        let mut total_uncompressed_size = 0;

        // 遍历子节点
        for child in &mut node.children {
            apply_dir_size(child);
            total_compressed_size += child.compressed_size;
            total_uncompressed_size += child.uncompressed_size;
        }

        // 更新当前目录的大小
        node.compressed_size = total_compressed_size;
        node.uncompressed_size = total_uncompressed_size;
    }
}

/// 合并唯一的嵌套目录为 a/b/c 形式的单个节点，减少可视化显示层级
fn merge_nested_dirs(nodes: &mut [RenderTreeNode]) {
    for node in nodes {
        merge_nested_dirs(&mut node.children);

        if node.is_dir && node.children.len() == 1 {
            let child = node.children.iter_mut().next().unwrap();
            if child.is_dir {
                let new_name = format!("{} / {}", node.name, child.name);
                // 合并后的节点
                node.name = new_name.clone();
                node.children = child.children.clone();
            }
        }
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
