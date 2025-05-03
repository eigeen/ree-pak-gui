import { invoke } from '@tauri-apps/api/core'

export function openSite(url: string): Promise<void> {
  return invoke('open_site', { url })
}
