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
  isCompressed: boolean
  belongsTo?: PakId
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

type WorkProgressEventImpl<T> =
  | {
      event: 'workStart'
      data: {
        count: number
      }
    }
  | {
      event: 'workFinished'
    }
  | {
      event: 'fileDone'
      data: T
    }
  | {
      event: 'error'
      data: {
        error: string
      }
    }

type UnpackProgressData = {
  path: string
  hash: JsSafeHash
  finishCount: number
}

type PackProgressData = {
  path: string
  finishCount: number
}

export type UnpackProgressEvent = WorkProgressEventImpl<UnpackProgressData>
export type PackProgressEvent = WorkProgressEventImpl<PackProgressData>

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

export function pak_extract_all(
  options: ExtractOptions,
  onEvent: Channel<UnpackProgressEvent>
): Promise<void> {
  return invoke('pak_extract_all', { options, onEvent })
}

export function pak_terminate_extraction(): Promise<void> {
  return invoke('pak_terminate_extraction')
}

export function pak_read_file_tree(): Promise<FileTree> {
  return invoke('pak_read_file_tree')
}

export function pak_read_file_tree_optimized(options?: RenderTreeOptions): Promise<RenderTreeNode> {
  return invoke('pak_read_file_tree_optimized', { options })
}

// Pack related APIs

export interface PakHeaderInfo {
  header: PakHeader
  entries: PakEntry[]
}

export interface PakHeader {
  magic: string
  majorVersion: number
  minorVersion: number
  feature: number
  totalFiles: number
  hash: string
  unkU32Sig: string
}

export interface PakEntry {
  hashNameLower: number
  hashNameUpper: number
  offset: number
  compressedSize: number
  uncompressedSize: number
  compressionType: number
  encryptionType: string
  checksum: number
  unkAttr: number
}

// Get pak file header information
export function pak_get_header(pakPath: string): Promise<PakHeaderInfo> {
  return invoke('pak_get_header', { pakPath })
}

// Pack files/folders
export function pak_pack(
  sources: string[],
  output: string,
  onEvent: Channel<PackProgressEvent>
): Promise<void> {
  return invoke('pak_pack', { sources, output, onEvent })
}

// Terminate pack operation
export function pak_terminate_pack(): Promise<void> {
  return invoke('pak_terminate_pack')
}
