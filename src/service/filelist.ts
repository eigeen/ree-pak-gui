/**
 * @file filelist.ts
 * File list service.
 * Each file has an identifier, mostly the file name without extension.
 * Identifier are unique within a source.
 */

import { FileListAPI, type FileListInfo } from '@/api/http/filelist'
import { fetchWithSpeedCheck } from '@/lib/http/download'
import { getFileListDir } from '@/lib/localDir'
import { getFileStem } from '@/utils/path'
import { join } from '@tauri-apps/api/path'
import { exists, mkdir, readDir, writeFile } from '@tauri-apps/plugin-fs'

export type SourceType = 'local' | 'remote'

export interface FileListSource {
  identifier: string
  sourceType: SourceType
  filePath: string
}

// interface FileListMetadata {
//   file_name: string
//   tags: string[]
//   description: string
// }

export class FileListService {
  private localSource: { [identifier: string]: FileListSource } = {}
  private remoteManifest: { [fileName: string]: FileListInfo } = {}
  private remoteServers: string[] = []

  /**
   * Get mixed sources of local and remote sources.
   * Local sources will override remote sources with the same identifier.
   */
  public getAllSource(): FileListSource[] {
    const sourcesMap: { [identifier: string]: FileListSource } = {}
    for (const identifier in this.localSource) {
      sourcesMap[identifier] = this.localSource[identifier]
    }
    for (const fileName in this.remoteManifest) {
      const source = this.remoteManifest[fileName]
      const identifier = getFileStem(source.file_name)
      if (identifier in sourcesMap) {
        console.warn(
          `Duplicate identifier ${identifier} found in local and remote sources, using local source.`
        )
        continue
      }
      sourcesMap[identifier] = {
        identifier,
        sourceType: 'remote',
        filePath: fileName
      }
    }

    // order by identifier
    const sources = Object.values(sourcesMap)
    sources.sort((a, b) => a.identifier.localeCompare(b.identifier))

    return sources
  }

  public async refreshLocalSource(): Promise<void> {
    const sources: { [identifier: string]: FileListSource } = {}

    const fileListDir = await getFileListDir(true)
    for (const file of await readDir(fileListDir)) {
      if (!file.isFile) continue

      const filePath = await join(fileListDir, file.name)
      const identifier = getFileStem(file.name)
      if (!identifier) {
        console.warn(`Invalid file name as identifier: ${file.name}, skipping.`)
        continue
      }

      sources[identifier] = {
        identifier,
        sourceType: 'local',
        filePath
      }
    }

    this.localSource = sources
  }

  public async fetchRemoteSource(): Promise<void> {
    const filelistApi = FileListAPI.getInstance()
    const manifest = await filelistApi.fetchFileListManifest()

    const sources: { [fileName: string]: FileListInfo } = {}
    for (const source of manifest.files) {
      sources[source.file_name] = source
    }

    this.remoteManifest = sources
    this.remoteServers = manifest.base_urls
  }

  public async downloadRemoteFile(fileName: string): Promise<void> {
    if (!this.remoteManifest[fileName]) {
      throw new Error(`File not found in remote source: ${fileName}`)
    }

    const info = this.remoteManifest[fileName]

    let lastError: any = null
    let blob: Blob | null = null
    for (const baseUrl of this.remoteServers) {
      try {
        const url = `${baseUrl}/${info.file_name}`
        blob = await fetchWithSpeedCheck(url, { connectTimeout: 5000 })
        break
      } catch (err) {
        lastError = err
        continue
      }
    }
    if (lastError) {
      throw new Error(`Failed to download file list from all sources: ${lastError}`)
    }
    if (!blob) {
      throw new Error('Failed to download file list from all sources: no response')
    }

    const targetDir = await this.getRemoteFileListDir()
    const targetPath = await join(targetDir, info.file_name)
    await writeFile(targetPath, new Uint8Array(await blob.arrayBuffer()))
  }

  private async getRemoteFileListDir(): Promise<string> {
    const filelistDir = await getFileListDir(false)
    const remoteDir = await join(filelistDir, 'remote')
    if (!(await exists(remoteDir))) {
      await mkdir(remoteDir, { recursive: true })
    }

    return remoteDir
  }
}
