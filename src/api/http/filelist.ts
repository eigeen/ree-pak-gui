import { fetch } from '@tauri-apps/plugin-http'

const MANIFEST_URLS = [
  'https://raw.githubusercontent.com/eigeen/ree-pak-gui-update/refs/heads/main/filelist_manifest.json',
  'https://gitee.com/eigeen/ree-pak-gui-update/raw/main/filelist_manifest.json'
]

export interface FileListManifest {
  base_urls: string[]
  files: FileListInfo[]
}

export interface FileListInfo {
  file_name: string
  tags: string[]
  update_time: string
  description: string
  size: number
  sha256: string
}

export class FileListAPI {
  private static instance: FileListAPI | null = null

  private constructor() {}

  public static getInstance(): FileListAPI {
    if (!FileListAPI.instance) {
      FileListAPI.instance = new FileListAPI()
    }
    return FileListAPI.instance
  }

  public async fetchFileListManifest(): Promise<FileListManifest> {
    let lastError: any = null
    for (const url of MANIFEST_URLS) {
      try {
        const response = await fetch(url, { method: 'GET', connectTimeout: 5000 })
        if (response.status !== 200) {
          throw new Error(
            `Failed to fetch file list manifest: ${response.status} ${response.statusText}`
          )
        }

        const manifest: FileListManifest = await response.json()
        return manifest
      } catch (err) {
        lastError = err
        continue
      }
    }

    throw new Error(`Failed to fetch file list manifest from all sources: ${lastError}`)
  }
}
