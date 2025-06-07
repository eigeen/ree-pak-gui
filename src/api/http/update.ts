import { fetch } from '@tauri-apps/plugin-http'

const METADATA_URLS = [
  'https://raw.githubusercontent.com/eigeen/ree-pak-gui-update/refs/heads/main/update.json',
  'https://gitee.com/eigeen/ree-pak-gui-update/raw/main/update.json'
]

export interface UpdateMetadata {
  versions: UpdateVersion[]
}

export interface UpdateVersion {
  version: string
  channel: string
  pub_time: string
  description?: string
  min_version?: string
  files: UpdateFile[]
}

export interface UpdateFile {
  name: string
  size: number
  sha256: string
  urls: string[]
}

export class UpdateAPI {
  private static instance: UpdateAPI | null = null

  private constructor() {}

  public static getInstance(): UpdateAPI {
    if (!UpdateAPI.instance) {
      UpdateAPI.instance = new UpdateAPI()
    }
    return UpdateAPI.instance
  }

  public async fetchUpdateMetadata(): Promise<UpdateMetadata> {
    let lastError: Error | null = null

    for (const url of METADATA_URLS) {
      try {
        const response = await fetch(url, { method: 'GET' })
        if (response.status === 200) {
          return (await response.json()) as UpdateMetadata
        } else {
          lastError = new Error(
            `Failed to fetch update metadata from ${url}: ${response.status} ${response.statusText}`
          )
          continue
        }
      } catch (e) {
        lastError = e as Error
      }
    }

    if (lastError) {
      throw new Error(`Failed to fetch update metadata from all sources: ${lastError.message}`)
    }
    throw new Error('Failed to fetch update metadata from all sources')
  }
}
