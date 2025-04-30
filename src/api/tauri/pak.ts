/// See src-tauri/src/command.rs for documentation.

import { Channel, invoke } from '@tauri-apps/api/core'

export type PakId = string
export type JsSafeHash = [number, number]

export interface PakInfo {
  id: PakId
  path: string
}

export interface FileTree {
  root: FileTreeNode
  uncompressedSize: number
  compressedSize: number
  fileCount: number
}

export interface FileTreeNode {
  info: NodeInfo
  children: { [key: string]: FileTreeNode }
}

export interface NodeInfo {
  isDir: boolean
  relativePath: string
  hash?: JsSafeHash
  uncompressedSize: number
  compressedSize: number
}

export interface RenderTreeOptions {
  mergeDirectories?: boolean
  sortByName?: boolean
  sortBySize?: boolean
}

export interface RenderTreeNode {
  isDir: boolean
  name: string
  hash?: JsSafeHash
  compressedSize: number
  uncompressedSize: number
  belongingTo?: PakId
  children: RenderTreeNode[]
}

export interface ExtractOptions {
  outputPath: string
  override: boolean
  extractAll: boolean
  extractFiles: ExtractFileInfo[]
}

export interface ExtractFileInfo {
  hash: JsSafeHash
  belongsTo: PakId
}

export type WorkProgressEvent =
  | {
      event: 'workStart'
      data: {
        fileCount: number
      }
    }
  | {
      event: 'workFinished'
      data: null
    }
  | {
      event: 'fileStart'
      data: {
        path: string
        hash: JsSafeHash
      }
    }
  | {
      event: 'fileDone'
      data: {
        hash: JsSafeHash
        finishCount: number
        errMsg?: string
      }
    }

export function pak_clear_all(): Promise<void> {
  return invoke('pak_clear_all')
}

export function pak_list_all(): Promise<PakInfo[]> {
  return invoke('pak_list_all')
}

export function pak_open(path: string): Promise<PakId> {
  return invoke('pak_open', { path })
}

export function pak_close(id: PakId): Promise<void> {
  return invoke('pak_close', { id })
}

export function pak_order(order: PakId[]): Promise<void> {
  return invoke('pak_order', { order })
}

export function pak_get_info(id: PakId): Promise<PakInfo> {
  return invoke('pak_get_info', { id })
}

export function pak_read_file_tree(): Promise<FileTree> {
  return invoke('pak_read_file_tree')
}

export function pak_read_file_tree_optimized(options?: RenderTreeOptions): Promise<RenderTreeNode> {
  return invoke('pak_read_file_tree_optimized', { options })
}

export function pak_extract_all(
  options: ExtractOptions,
  onEvent: Channel<WorkProgressEvent>
): Promise<void> {
  return invoke('pak_extract_all', { options, onEvent })
}
