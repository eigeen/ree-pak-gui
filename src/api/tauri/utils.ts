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

export interface ModelInsightStatus {
  installed: boolean
  platform: string
  arch: string
  installDir: string
  expectedPath: string
  executablePath?: string
}

export interface ModelInsightOpenMeshOptions {
  hash: JsSafeHash
  belongsTo?: string
  entryPath: string
}

export interface ModelInsightRenderMeshOptions extends ModelInsightOpenMeshOptions {
  width?: number
  height?: number
  startResidentViewer?: boolean
}

export interface ModelInsightLaunchInfo {
  sessionId: string
  manifestPath: string
  rpcAddr: string
  executablePath: string
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

export function performUpdate(filePath: string): Promise<void> {
  return invoke('perform_update', { filePath })
}

export function vgmstreamGetStatus(): Promise<VgmstreamStatus> {
  return invoke('vgmstream_get_status')
}

export function modelInsightGetStatus(): Promise<ModelInsightStatus> {
  return invoke('model_insight_get_status')
}

export function modelInsightOpenMesh(
  options: ModelInsightOpenMeshOptions
): Promise<ModelInsightLaunchInfo> {
  return invoke('model_insight_open_mesh', { options })
}

export function modelInsightRenderMeshPreview(
  options: ModelInsightRenderMeshOptions
): Promise<string> {
  return invoke('model_insight_render_mesh_preview', { options })
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
