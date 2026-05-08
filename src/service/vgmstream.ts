import { join } from '@tauri-apps/api/path'
import { exists, mkdir, writeFile } from '@tauri-apps/plugin-fs'
import {
  VGMSTREAM_FALLBACK_TAG,
  VgmstreamAPI,
  type VgmstreamRelease,
  type VgmstreamReleaseAsset
} from '@/api/http/vgmstream'
import {
  vgmstreamGetStatus,
  vgmstreamInstallFromArchive,
  type VgmstreamStatus
} from '@/api/tauri/utils'
import { fetchWithSpeedCheck } from '@/lib/http/download'
import { getDownloadingDir } from '@/lib/localDir'
import { logFrontendDebug, runLoggedTask } from '@/utils/frontendLog'

type VgmstreamReleaseSource = 'latest' | 'fallback'

export interface VgmstreamInstallResult {
  status: VgmstreamStatus
  release: VgmstreamRelease
  asset: VgmstreamReleaseAsset
  source: VgmstreamReleaseSource
}

export class VgmstreamService {
  private static instance: VgmstreamService | null = null

  private api = VgmstreamAPI.getInstance()

  private constructor() {}

  public static getInstance(): VgmstreamService {
    if (!VgmstreamService.instance) {
      VgmstreamService.instance = new VgmstreamService()
    }
    return VgmstreamService.instance
  }

  public async getStatus(): Promise<VgmstreamStatus> {
    return await vgmstreamGetStatus()
  }

  public async downloadAndInstall(
    onEvent?: (event: ProgressEvent) => Promise<void>
  ): Promise<VgmstreamInstallResult> {
    return await runLoggedTask(
      'vgmstream.install',
      async () => {
        const status = await this.getStatus()
        const assetName = status.assetName
        if (!assetName) {
          throw new Error(`Unsupported platform: ${status.platform}-${status.arch}`)
        }

        const target = await this.resolveTargetAsset(assetName)
        const archivePath = await this.downloadAsset(target.asset, onEvent)
        const installedStatus = await vgmstreamInstallFromArchive(archivePath)

        return {
          status: installedStatus,
          release: target.release,
          asset: target.asset,
          source: target.source
        }
      },
      {
        start: 'download vgmstream',
        success: ({ release, asset, source }) =>
          `installed asset=${asset.name} release=${release.tagName} source=${source}`
      }
    )
  }

  private async resolveTargetAsset(assetName: string): Promise<{
    release: VgmstreamRelease
    asset: VgmstreamReleaseAsset
    source: VgmstreamReleaseSource
  }> {
    const latest = await this.api.fetchLatestRelease()
    const latestAsset = this.findAsset(latest, assetName)
    if (latestAsset) {
      return {
        release: latest,
        asset: latestAsset,
        source: 'latest'
      }
    }

    logFrontendDebug(
      'vgmstream.install',
      `asset=${assetName} not found in latest=${latest.tagName}; fallback=${VGMSTREAM_FALLBACK_TAG}`
    )
    const fallback = await this.api.fetchFallbackRelease()
    const fallbackAsset = this.findAsset(fallback, assetName)
    if (!fallbackAsset) {
      throw new Error(
        `No ${assetName} asset found in latest release ${latest.tagName} or fallback ${VGMSTREAM_FALLBACK_TAG}.`
      )
    }

    return {
      release: fallback,
      asset: fallbackAsset,
      source: 'fallback'
    }
  }

  private findAsset(release: VgmstreamRelease, assetName: string) {
    return release.assets.find((asset) => asset.name === assetName)
  }

  private async downloadAsset(
    asset: VgmstreamReleaseAsset,
    onEvent?: (event: ProgressEvent) => Promise<void>
  ) {
    const downloadDir = await getDownloadingDir(true)
    const extensionDir = await join(downloadDir, 'vgmstream')
    if (!(await exists(extensionDir))) {
      await mkdir(extensionDir, { recursive: true })
    }

    const archivePath = await join(extensionDir, asset.name)
    const blob = await fetchWithSpeedCheck(asset.url, {}, onEvent)
    await writeFile(archivePath, new Uint8Array(await blob.arrayBuffer()))
    return archivePath
  }
}
