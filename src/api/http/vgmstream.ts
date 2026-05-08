import { fetch } from '@tauri-apps/plugin-http'

const VGMSTREAM_RELEASES_API_URL = 'https://api.github.com/repos/vgmstream/vgmstream/releases'
export const VGMSTREAM_RELEASES_URL = 'https://github.com/vgmstream/vgmstream/releases'
export const VGMSTREAM_FALLBACK_TAG = 'r2083'

interface GithubReleaseAsset {
  name: string
  size: number
  state: string
  browser_download_url: string
}

interface GithubRelease {
  tag_name: string
  name: string | null
  html_url: string
  draft: boolean
  prerelease: boolean
  published_at: string | null
  assets: GithubReleaseAsset[]
}

export interface VgmstreamReleaseAsset {
  name: string
  size: number
  url: string
}

export interface VgmstreamRelease {
  tagName: string
  name?: string
  releaseUrl: string
  publishedAt?: string
  assets: VgmstreamReleaseAsset[]
}

function normalizeRelease(release: GithubRelease): VgmstreamRelease {
  return {
    tagName: release.tag_name,
    name: release.name ?? undefined,
    releaseUrl: release.html_url,
    publishedAt: release.published_at ?? undefined,
    assets: release.assets
      .filter((asset) => asset.state === 'uploaded' && asset.browser_download_url)
      .map((asset) => ({
        name: asset.name,
        size: asset.size,
        url: asset.browser_download_url
      }))
  }
}

export class VgmstreamAPI {
  private static instance: VgmstreamAPI | null = null

  private constructor() {}

  public static getInstance(): VgmstreamAPI {
    if (!VgmstreamAPI.instance) {
      VgmstreamAPI.instance = new VgmstreamAPI()
    }
    return VgmstreamAPI.instance
  }

  public async fetchLatestRelease(): Promise<VgmstreamRelease> {
    return await this.fetchRelease(`${VGMSTREAM_RELEASES_API_URL}/latest`)
  }

  public async fetchFallbackRelease(): Promise<VgmstreamRelease> {
    return await this.fetchRelease(`${VGMSTREAM_RELEASES_API_URL}/tags/${VGMSTREAM_FALLBACK_TAG}`)
  }

  private async fetchRelease(url: string): Promise<VgmstreamRelease> {
    let response: Awaited<ReturnType<typeof fetch>>
    try {
      response = await fetch(url, {
        method: 'GET',
        connectTimeout: 15000,
        headers: {
          Accept: 'application/vnd.github+json',
          'X-GitHub-Api-Version': '2022-11-28'
        }
      })
    } catch (error) {
      throw new Error(`Failed to fetch vgmstream release from GitHub: ${String(error)}`)
    }

    if (response.status !== 200) {
      throw new Error(
        `Failed to fetch vgmstream release from GitHub: ${response.status} ${response.statusText}`
      )
    }

    const release = (await response.json()) as GithubRelease
    if (release.draft || release.prerelease) {
      throw new Error(`vgmstream release is not installable: ${release.tag_name}`)
    }

    return normalizeRelease(release)
  }
}
