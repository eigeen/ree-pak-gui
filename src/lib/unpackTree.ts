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
  return nodes
    .map((node) => pruneTreeFiles(node))
    .filter((node): node is TreeData => node !== null)
}

export function createTreeFilter(filterText = '', regexMode = false): string | RegExp {
  const filter = filterText.trim()
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
  return data
    .map((node) => {
      const filteredChildren = filterTreeData(node.children, filter)
      const isMatch =
        typeof filter === 'string'
          ? filter === '' || node.path.toLowerCase().includes(filter)
          : filter.test(node.path)

      if (!isMatch && filteredChildren.length === 0) {
        return null
      }

      return {
        ...node,
        children: filteredChildren
      }
    })
    .filter((node): node is TreeData => node !== null)
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

function pruneTreeFiles(node: TreeData): TreeData | null {
  if (!node.isDir) {
    return null
  }

  return {
    ...node,
    children: node.children
      .map((child) => pruneTreeFiles(child))
      .filter((child): child is TreeData => child !== null)
  }
}
