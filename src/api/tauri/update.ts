import { Channel, invoke } from '@tauri-apps/api/core'

export interface AppUpdateInfo {
  version: string
  name: string
  date: string
  releaseMarkdown?: string
}

export type AppUpdateProgressEvent =
  | { event: 'checking' }
  | { event: 'downloadStarted'; data: { total: number | null } }
  | { event: 'downloadProgress'; data: { downloaded: number; total: number | null } }
  | { event: 'installing' }
  | { event: 'relaunching' }
  | { event: 'finished'; data: { version: string } }
  | { event: 'upToDate' }
  | { event: 'error'; data: { error: string } }

export function appCheckUpdate(): Promise<AppUpdateInfo | null> {
  return invoke('app_check_update')
}

export function appInstallUpdate(onEvent: Channel<AppUpdateProgressEvent>): Promise<void> {
  return invoke('app_install_update', { onEvent })
}
