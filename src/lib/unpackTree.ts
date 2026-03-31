import type { JsSafeHash, RenderTreeNode } from '@/api/tauri/pak'

export interface TreeData {
  id: string
  name: string
  label: string
  path: string
  parentId?: string
  hash?: JsSafeHash
  isDir: boolean
  compressedSize: number
  uncompressedSize: number
  isCompressed: boolean
  sizeText: string
  children: TreeData[]
  belongsTo: string | undefined
}

export function buildTreeData(nodes: RenderTreeNode[]): TreeData[] {
  return nodes.map((node) => createTreeData(node))
}

export function buildDirectoryTreeData(nodes: TreeData[]): TreeData[] {
  return mapTreeNodes(nodes, pruneTreeFiles)
}

export function createTreeFilter(filterText = '', regexMode = false): string | RegExp {
  const filter = filterText.trim()
  if (filter === '') {
    return ''
  }

  if (!regexMode) {
    return filter.toLowerCase()
  }

  try {
    return new RegExp(filter, 'i')
  } catch {
    return filter.toLowerCase()
  }
}

export function filterTreeData(data: TreeData[], filter: string | RegExp): TreeData[] {
  if (typeof filter === 'string' && filter === '') {
    return data
  }

  return mapTreeNodes(data, (node) => {
    const filteredChildren = filterTreeData(node.children, filter)
    const isMatch =
      typeof filter === 'string' ? node.path.toLowerCase().includes(filter) : filter.test(node.path)

    if (!isMatch && filteredChildren.length === 0) {
      return null
    }

    if (filteredChildren === node.children) {
      return node
    }

    return {
      ...node,
      children: filteredChildren
    }
  })
}

function createTreeData(node: RenderTreeNode, parentPath = '', parentId?: string): TreeData {
  const id = node.hash ? node.hash.toString() : `${parentPath}/${node.name}`
  const path = parentPath ? `${parentPath}/${node.name}` : node.name

  return {
    id,
    name: node.name,
    label: node.name,
    path,
    parentId,
    hash: node.hash,
    isDir: node.isDir,
    compressedSize: node.compressedSize,
    uncompressedSize: node.uncompressedSize,
    isCompressed: node.isCompressed,
    sizeText: formatTreeSize(
      node.uncompressedSize !== undefined
        ? node.isCompressed
          ? node.uncompressedSize
          : node.compressedSize
        : 0
    ),
    children: node.children?.map((child) => createTreeData(child, path, id)) ?? [],
    belongsTo: node.belongsTo
  }
}

function formatTreeSize(size: number): string {
  if (size < 0) return 'Invalid'

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let index = 0
  let current = size

  while (current >= 1024 && index < units.length - 1) {
    current /= 1024
    index++
  }

  return `${current.toFixed(2)} ${units[index]}`
}

function mapTreeNodes(
  nodes: TreeData[],
  mapNode: (node: TreeData) => TreeData | null
): TreeData[] {
  const nextNodes: TreeData[] = []
  let changed = false

  for (const node of nodes) {
    const nextNode = mapNode(node)
    if (!nextNode) {
      changed = true
      continue
    }

    nextNodes.push(nextNode)
    if (nextNode !== node) {
      changed = true
    }
  }

  return changed ? nextNodes : nodes
}

function pruneTreeFiles(node: TreeData): TreeData | null {
  if (!node.isDir) {
    return null
  }

  const directoryChildren = mapTreeNodes(node.children, pruneTreeFiles)
  if (directoryChildren === node.children) {
    return node
  }

  return {
    ...node,
    children: directoryChildren
  }
}
