import { invoke } from '@tauri-apps/api/core'

export interface UpdateVersion {
  version: string
  channel: UpdateChannel
  pub_time: string
  min_version?: string
  files: UpdateFile[]
}

export interface UpdateFile {
  name: string
  size: number
  sha256: string
}

export enum UpdateChannel {
  Release,
  Nightly
}

export class Update {
  static check(): Promise<UpdateVersion> {
    return invoke('update_check')
  }

  static perform(updateVersion: UpdateVersion): Promise<void> {
    return invoke('update_perform', { updateVersion })
  }
}
