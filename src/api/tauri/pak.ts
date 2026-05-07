/// See src-tauri/src/command.rs for documentation.

import { Channel, invoke } from '@tauri-apps/api/core'

export type PakId = string
export type JsSafeHash = [number, number]

export interface PakInfo {
  id: PakId
  path: string
}

export interface FileTree {
  roots: FileTreeNode[]
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
  mode: ExtractMode
  extractAll: boolean
  extractFiles: ExtractFileInfo[]
}

export type ExtractMode = 'relativePath' | 'absolutePath'

export interface ExtractFileInfo {
  hash: JsSafeHash
  belongsTo: PakId
  relativeRoot?: string
}

export type WorkProgressEvent<T> =
  | {
      event: 'workStart'
      data: {
        count: number
      }
    }
  | {
      event: 'workFinished'
      data?: T
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
  tree?: PackedFileTree
}

type PackedFileTree = {
  paks: Array<PackedPak>
}

export type PackedPak = {
  path: string
  files: Array<{
    path: string
    hash: JsSafeHash
    size: number
  }>
}

export type UnpackProgressEvent = WorkProgressEvent<UnpackProgressData>
export type PackProgressEvent = WorkProgressEvent<PackProgressData>

export type PackConflictResolution = Record<string, string | null>

export interface PackAnalyzeOptions {
  sources: string[]
  allowFileNameAsPathHash: boolean
}

export interface PackConflictSourceInfo {
  id: string
  sourcePath: string
}

export interface PackConflictInfo {
  targetKey: string
  targetPath: string
  size?: number
  modifiedTimestampMs?: number | null
  sources: PackConflictSourceInfo[]
  selectedSourceId?: string | null
}

export interface PackOptions {
  sources: string[]
  output: string
  allowFileNameAsPathHash: boolean
  conflictResolutions?: PackConflictResolution
}

export interface AudioSourceRef {
  hash: JsSafeHash
  belongsTo: PakId
}

export interface AudioExtractBatchOptions {
  source: AudioSourceRef
  indices: number[]
  outputDir?: string
}

export type AudioContainerKind = 'bnk' | 'pck'

export interface AudioEntryInfo {
  index: number
  wemId: number
  offset: number
  size: number
  languageId?: number | null
}

export interface AudioContainerInfo {
  sourcePath: string
  containerKind: AudioContainerKind
  entries: AudioEntryInfo[]
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

export function pak_read_file_tree_optimized(
  options?: RenderTreeOptions
): Promise<RenderTreeNode[]> {
  return new Promise((resolve, reject) => {
    let settled = false
    const onEvent = new Channel<WorkProgressEvent<RenderTreeNode[]>>()

    onEvent.onmessage = (event) => {
      if (settled) return

      switch (event.event) {
        case 'workFinished':
          settled = true
          resolve(event.data ?? [])
          break
        case 'error':
          settled = true
          reject(new Error(event.data.error))
          break
      }
    }

    invoke('pak_read_file_tree_optimized', { options, onEvent }).catch((error) => {
      if (settled) return
      settled = true
      reject(error)
    })
  })
}

// Pack related APIs

export interface PakHeaderInfo {
  header: PakHeader
  entries: PakEntry[]
}

export interface PakHeader {
  magic: number[]
  majorVersion: number
  minorVersion: number
  feature: number
  totalFiles: number
  hash: string
  unkU32Sig: number
}

export interface PakEntry {
  hashNameLower: number
  hashNameUpper: number
  offset: number
  compressedSize: number
  uncompressedSize: number
  compressionType: number
  encryptionType: string
  checksum: string
  unkAttr: string
}

// Get pak file header information
export function pak_get_header(pakPath: string): Promise<PakHeaderInfo> {
  return invoke('pak_get_header', { pakPath })
}

export function pak_analyze_conflicts(options: PackAnalyzeOptions): Promise<PackConflictInfo[]> {
  return invoke('pak_analyze_conflicts', { options })
}

// Pack files/folders
export function pak_pack(options: PackOptions, onEvent: Channel<PackProgressEvent>): Promise<void> {
  return invoke('pak_pack', { options, onEvent })
}

// Terminate pack operation
export function pak_terminate_pack(): Promise<void> {
  return invoke('pak_terminate_pack')
}

export function audio_list_container(source: AudioSourceRef): Promise<AudioContainerInfo> {
  return invoke('audio_list_container', { source })
}

export function audio_extract_wems(options: AudioExtractBatchOptions): Promise<string[]> {
  return invoke('audio_extract_wems', { options })
}

export function audio_extract_wavs(options: AudioExtractBatchOptions): Promise<string[]> {
  return invoke('audio_extract_wavs', { options })
}
