import { fetch } from '@tauri-apps/plugin-http'

const MANIFEST_URLS = [
  'https://raw.githubusercontent.com/eigeen/ree-pak-gui-update/refs/heads/main/filelist_manifest.json',
  'https://gitee.com/eigeen/ree-pak-gui-update/raw/main/filelist_manifest.json'
]

const MANIFEST_CONNECT_TIMEOUT_MS = 5000
// Hard timeout to avoid fetch promise hanging forever (e.g. blocked network / DNS / TLS stall).
const MANIFEST_REQUEST_TIMEOUT_MS = 8000
const MANIFEST_MAX_REDIRECTIONS = 5

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

  private createManifestFetchTask(url: string) {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => {
      controller.abort(`Timeout fetching manifest: ${url}`)
    }, MANIFEST_REQUEST_TIMEOUT_MS)

    const promise = (async (): Promise<FileListManifest> => {
      try {
        const response = await fetch(url, {
          method: 'GET',
          connectTimeout: MANIFEST_CONNECT_TIMEOUT_MS,
          maxRedirections: MANIFEST_MAX_REDIRECTIONS,
          signal: controller.signal
        })
        if (response.status !== 200) {
          throw new Error(
            `Failed to fetch file list manifest: ${response.status} ${response.statusText}`
          )
        }

        return (await response.json()) as FileListManifest
      } finally {
        clearTimeout(timeoutId)
      }
    })()

    return {
      url,
      promise,
      abort: () => controller.abort(`Cancelled: ${url}`)
    }
  }

  public async fetchFileListManifest(): Promise<FileListManifest> {
    const tasks = MANIFEST_URLS.map((url) => this.createManifestFetchTask(url))

    return await new Promise<FileListManifest>((resolve, reject) => {
      let pending = tasks.length
      let lastError: unknown = null
      let resolved = false

      for (const task of tasks) {
        task.promise
          .then((manifest) => {
            if (resolved) return
            resolved = true
            for (const t of tasks) {
              if (t.url !== task.url) t.abort()
            }
            resolve(manifest)
          })
          .catch((err) => {
            lastError = err
            pending -= 1
            if (pending === 0 && !resolved) {
              reject(
                new Error(`Failed to fetch file list manifest from all sources: ${String(lastError)}`)
              )
            }
          })
      }
    })
  }
}
