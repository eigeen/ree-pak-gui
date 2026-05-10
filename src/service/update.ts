import { appCheckUpdate, appInstallUpdate } from '@/api/tauri/update'
import type { AppUpdateInfo, AppUpdateProgressEvent } from '@/api/tauri/update'
import { logFrontendDebug, runLoggedTask } from '@/utils/frontendLog'

import { Channel } from '@tauri-apps/api/core'

export type UpdateProgressEvent = AppUpdateProgressEvent

export class UpdateService {
  private static instance: UpdateService | null = null

  private targetVersion: AppUpdateInfo | null = null

  public static getInstance(): UpdateService {
    if (!UpdateService.instance) {
      UpdateService.instance = new UpdateService()
    }
    return UpdateService.instance
  }

  public async initialize(): Promise<void> {
    // Kept as a no-op for callers that initialize services eagerly.
  }

  public async checkForUpdates(): Promise<AppUpdateInfo | null> {
    return runLoggedTask(
      'update.check',
      async () => {
        this.targetVersion = await appCheckUpdate()
        if (this.targetVersion) {
          logFrontendDebug(
            'update.check',
            `update available version=${this.targetVersion.version}`
          )
        }
        return this.targetVersion
      },
      {
        start: 'check github releases via backend',
        success: (version) =>
          version ? `update available version=${version.version}` : 'no update available'
      }
    )
  }

  public async installUpdate(onEvent?: (event: UpdateProgressEvent) => Promise<void> | void) {
    await runLoggedTask(
      'update.install',
      async () => {
        const channel = new Channel<UpdateProgressEvent>()
        channel.onmessage = (event) => {
          void onEvent?.(event)
        }

        await appInstallUpdate(channel)
      },
      {
        start: this.targetVersion
          ? `install version=${this.targetVersion.version}`
          : 'install update',
        success: 'backend update finished'
      }
    )
  }
}
