/// Path scanner utility class for managing scan operations

import { Channel } from '@tauri-apps/api/core'
import { scanPaths, terminatePathScan } from '@/api/tauri/tools'
import type { PathScanOptions, PathScanProgressEvent } from '@/api/tauri/tools'

export class PathScanner {
  private channel: Channel<PathScanProgressEvent> | null = null

  constructor(private progressCallback: (progress: PathScanProgressEvent) => void) {}

  async scan(options: PathScanOptions): Promise<void> {
    try {
      // Create channel for progress updates
      this.channel = new Channel<PathScanProgressEvent>()

      // Listen for progress events
      this.channel.onmessage = (event) => {
        this.progressCallback(event)
      }

      // Start the scan operation
      await scanPaths(options, this.channel)
    } catch (e) {
      this.channel = null
      throw e
    }
  }

  async terminate(): Promise<void> {
    try {
      await terminatePathScan()
    } catch (error) {
      console.error('Failed to terminate path scan:', error)
    }
    this.channel = null
  }
}
