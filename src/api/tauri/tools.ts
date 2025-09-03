/// Tools API for path scanning and other utilities.

import { Channel, invoke } from '@tauri-apps/api/core'

export type PathScanProgressEvent =
  | {
      event: 'startFile'
      data: {
        current: number
        total: number
      }
    }
  | {
      event: 'finish'
      data: {
        success: boolean
        foundPaths: string[]
        error?: string
      }
    }

export interface PathScanOptions {
  pakFiles: string[]
  dumpFiles: string[]
}

/**
 * Start path scanning operation with progress updates
 */
export async function scanPaths(
  options: PathScanOptions,
  onEvent: Channel<PathScanProgressEvent>
): Promise<void> {
  return invoke('tools_scan_paths', { options, onEvent })
}

/**
 * Terminate ongoing path scanning operation
 */
export async function terminatePathScan(): Promise<void> {
  return invoke('tools_terminate_scan')
}
