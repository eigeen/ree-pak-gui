import { invoke } from '@tauri-apps/api/core'
import type { JsSafeHash } from './pak'

export interface CompileInfo {
  version: string
  commitTime: string
  commitHash: string
  platform: string
  arch: string
}

export function getPreviewFile(hash: JsSafeHash): Promise<string> {
  return invoke('get_preview_file', { hash })
}

export function getExePath(): Promise<string> {
  return invoke('get_exe_path')
}

export function getCompileInfo(): Promise<CompileInfo> {
  return invoke('get_compile_info')
}

export function performUpdate(filePath: string): Promise<void> {
  return invoke('perform_update', { filePath })
}

export function zipExtractFile(filePath: string, outputPath?: string): Promise<void> {
  return invoke('zip_extract_file', { filePath, outputPath })
}

export function murmur32(buffer: Uint8Array): Promise<number> {
  return invoke('murmur32', { buffer })
}

export function murmur32_utf16(str: string): Promise<number> {
  return invoke('murmur32_utf16', { str })
}
