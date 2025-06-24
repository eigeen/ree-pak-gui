/**
 * @file filelist.ts
 * File list service.
 * Each file has an identifier, mostly the file name without extension.
 * Identifier are unique within a source.
 */

import { FileListAPI, type FileListInfo } from '@/api/http/filelist'
import { zipExtractFile } from '@/api/tauri/utils'
import { fetchWithSpeedCheck } from '@/lib/http/download'
import { getFileListDir, getTempDir } from '@/lib/localDir'
import { NameListFile } from '@/lib/NameListFile'
import { useFileListStore } from '@/store/filelist'
import { getFileStem } from '@/utils/path'
import { join } from '@tauri-apps/api/path'
import { exists, mkdir, readDir, remove, writeFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { reactive, type Reactive } from 'vue'

export class FileListService {
  private static instance: FileListService | null = null
  private static readonly NOTIFY_FILE_NAME = '_DONT_EDIT_FILES'

  private store
  private remoteServers: string[] = []

  private constructor() {
    this.store = useFileListStore()
  }

  public static getInstance(): FileListService {
    if (!FileListService.instance) {
      FileListService.instance = new FileListService()
    }
    return FileListService.instance
  }

  public async refreshLocalSource(): Promise<void> {
    // local manual sources
    const sources: { [identifier: string]: Reactive<NameListFile> } = {}

    const fileListDir = await getFileListDir(true)
    for (const file of await readDir(fileListDir)) {
      if (!file.isFile) continue

      const filePath = await join(fileListDir, file.name)
      const identifier = getFileStem(file.name)
      if (!identifier) {
        console.warn(`Invalid file name as identifier: ${file.name}, skipping.`)
        continue
      }

      const listfile = reactive(new NameListFile(identifier, 'local', filePath))
      await listfile.loadMetadata()
      sources[identifier] = listfile
    }
    this.store.localFile = sources

    // local downloaded sources
    const dlSources: { [identifier: string]: Reactive<NameListFile> } = {}

    const downloadedDir = await FileListService.getDownloadedDir()
    for (const file of await readDir(downloadedDir)) {
      if (!file.isFile) continue
      if (file.name === FileListService.NOTIFY_FILE_NAME) continue

      const filePath = await join(downloadedDir, file.name)
      const identifier = getFileStem(file.name)
      if (!identifier) {
        console.warn(`Invalid file name as identifier: ${file.name}, skipping.`)
        continue
      }

      const listfile = reactive(new NameListFile(identifier, 'remote', filePath))
      await listfile.loadMetadata()
      dlSources[identifier] = listfile
    }
    this.store.downloadedFile = dlSources

    console.debug('refreshed local source', this.store.localFile, this.store.downloadedFile)
    await FileListService.touchNotifyFile()
  }

  public async fetchRemoteSource(): Promise<void> {
    const filelistApi = FileListAPI.getInstance()
    const manifest = await filelistApi.fetchFileListManifest()

    const sources: { [fileName: string]: FileListInfo } = {}
    for (const source of manifest.files) {
      sources[source.file_name] = source
    }

    this.store.remoteManifest = sources
    this.remoteServers = manifest.base_urls
  }

  public async downloadRemoteFile(fileName: string): Promise<void> {
    if (!this.store.remoteManifest[fileName]) {
      throw new Error(`File not found in remote source: ${fileName}`)
    }

    const info = this.store.remoteManifest[fileName]

    let lastError: any = null
    let blob: Blob | null = null
    for (const baseUrl of this.remoteServers) {
      try {
        const url = `${baseUrl}/${info.file_name}`
        console.debug(`Trying to download file list from ${url}`)
        blob = await fetchWithSpeedCheck(url, { connectTimeout: 5000 })
        lastError = null
        break
      } catch (err) {
        lastError = err
        blob = null
        console.error('Download failed for', baseUrl, err)
        continue
      }
    }
    if (lastError) {
      throw new Error(`Failed to download file list from all sources: ${lastError}`)
    }
    if (!blob) {
      throw new Error('Failed to download file list from all sources: no response')
    }

    // check if need to unzip
    const ext = info.file_name.split('.').pop()
    if (ext === 'zip') {
      // save to temp dir
      const tempDir = await getTempDir(true)
      const tempPath = await join(tempDir, info.file_name)
      await writeFile(tempPath, new Uint8Array(await blob.arrayBuffer()))
      // unzip file
      const targetDir = await FileListService.getDownloadedDir()
      await zipExtractFile(tempPath, targetDir)
      // check extracted file exists
      const extractedFile = await join(targetDir, info.file_name.replace('.zip', ''))
      if (!exists(extractedFile)) {
        throw new Error('Failed to extract file list from downloaded zip file')
      }
      return
    }

    // directly save
    const targetDir = await FileListService.getDownloadedDir()
    const targetPath = await join(targetDir, info.file_name)
    await writeFile(targetPath, new Uint8Array(await blob.arrayBuffer()))
  }

  public async removeDownloaded(identifier: string): Promise<void> {
    const file = this.store.downloadedFile[identifier]
    if (!file) {
      throw new Error(`File not found in downloaded source: ${identifier}`)
    }

    const filePath = file.source.filePath
    if (!(await exists(filePath))) {
      throw new Error(`File not found in downloaded directory: ${filePath}`)
    }

    console.log('Removing file:', filePath)
    await remove(filePath)
    delete this.store.downloadedFile[identifier]
  }

  public async removeLocal(identifier: string): Promise<void> {
    const file = this.store.localFile[identifier]
    if (!file) {
      throw new Error(`File not found in local source: ${identifier}`)
    }

    const filePath = file.source.filePath
    if (!(await exists(filePath))) {
      throw new Error(`File not found in local directory: ${filePath}`)
    }

    console.log('Removing file:', filePath)
    await remove(filePath)
    delete this.store.localFile[identifier]
  }

  public getFileByIdent(identifier: string): Reactive<NameListFile> | null {
    const file = this.store.localFile[identifier] || this.store.downloadedFile[identifier]
    if (!file) {
      return null
    }
    return file
  }

  public static async getDownloadedDir(): Promise<string> {
    const filelistDir = await getFileListDir(false)
    const remoteDir = await join(filelistDir, 'remote')
    if (!(await exists(remoteDir))) {
      await mkdir(remoteDir, { recursive: true })
    }

    return remoteDir
  }

  private static async touchNotifyFile(): Promise<void> {
    const path = await join(
      await FileListService.getDownloadedDir(),
      FileListService.NOTIFY_FILE_NAME
    )
    if (!(await exists(path))) {
      await writeTextFile(path, '')
    }
  }
}
