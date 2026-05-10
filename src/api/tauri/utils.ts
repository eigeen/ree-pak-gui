import { Channel, invoke } from '@tauri-apps/api/core'
import type { ExtractFileInfo, JsSafeHash, WorkProgressEvent } from './pak'

export interface CompileInfo {
  version: string
  commitTime: string
  commitHash: string
  platform: string
  arch: string
}

export interface VgmstreamStatus {
  installed: boolean
  platform: string
  arch: string
  assetName?: string
  installDir: string
  expectedPath: string
  executablePath?: string
}

export interface ModelInsightLoadMeshAssetsOptions {
  hash: JsSafeHash
  belongsTo?: string
  entryPath: string
}

export interface ModelInsightMeshAssets {
  meshEntryPath: string
  meshFileVersion: number
  meshData: number[] | Uint8Array
  mdfEntryPath?: string | null
  mdfFileVersion?: number | null
  mdfData?: number[] | Uint8Array | null
}

export interface ModelInsightLoadTexturePreviewsOptions {
  belongsTo?: string
  baseEntryPath: string
  texturePaths: string[]
  textureResolution?: ModelTextureResolution
}

export type ModelTextureResolution = 'standard' | 'high'

export interface ModelInsightTexturePreview {
  texturePath: string
  entryPath: string
  previewPath: string
  previewData: number[] | Uint8Array
}

export type TextureExportFormat = 'dds' | 'png'

export interface TextureExportOptions {
  outputPath: string
  format: TextureExportFormat
  files: ExtractFileInfo[]
}

export type TextureExportProgressData = {
  path: string
  finishCount: number
}

export type TextureExportProgressEvent = WorkProgressEvent<TextureExportProgressData>

export function getPreviewFile(hash: JsSafeHash): Promise<string> {
  return invoke('get_preview_file', { hash })
}

export function exportTextureFiles(
  options: TextureExportOptions,
  onEvent: Channel<TextureExportProgressEvent>
): Promise<number> {
  return invoke('export_texture_files', { options, onEvent })
}

export function terminateTextureExport(): Promise<void> {
  return invoke('terminate_texture_export')
}

export function getExePath(): Promise<string> {
  return invoke('get_exe_path')
}

export function getCompileInfo(): Promise<CompileInfo> {
  return invoke('get_compile_info')
}

export function vgmstreamGetStatus(): Promise<VgmstreamStatus> {
  return invoke('vgmstream_get_status')
}

export function modelInsightLoadMeshAssets(
  options: ModelInsightLoadMeshAssetsOptions
): Promise<ModelInsightMeshAssets> {
  return invoke('model_insight_load_mesh_assets', { options })
}

export function modelInsightLoadTexturePreviews(
  options: ModelInsightLoadTexturePreviewsOptions
): Promise<ModelInsightTexturePreview[]> {
  return invoke('model_insight_load_texture_previews', { options })
}

export function vgmstreamInstallFromArchive(archivePath: string): Promise<VgmstreamStatus> {
  return invoke('vgmstream_install_from_archive', { archivePath })
}

export function zipExtractFile(filePath: string, outputPath?: string): Promise<void> {
  return invoke('zip_extract_file', { filePath, outputPath })
}

export function murmur32(buffer: Uint8Array): Promise<number> {
  return invoke('murmur32', { buffer })
}

export function murmur32_utf16(str: string): Promise<JsSafeHash> {
  return invoke('murmur32_utf16', { str })
}
