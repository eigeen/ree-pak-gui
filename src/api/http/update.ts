import { fetch } from '@tauri-apps/plugin-http'

const GITHUB_RELEASES_API_URL =
  'https://api.github.com/repos/eigeen/ree-pak-gui/releases?per_page=20'

interface GithubReleaseAsset {
  name: string
  size: number
  state: string
  digest?: string | null
  browser_download_url: string
}

interface GithubRelease {
  tag_name: string
  html_url: string
  body: string | null
  draft: boolean
  prerelease: boolean
  published_at: string | null
  assets: GithubReleaseAsset[]
}

export interface UpdateMetadata {
  versions: UpdateVersion[]
}

export interface UpdateVersion {
  version: string
  tagName: string
  pub_time: string
  description?: string
  releaseUrl: string
  files: UpdateFile[]
}

export interface UpdateFile {
  name: string
  size: number
  sha256?: string
  url: string
}

function normalizeVersion(tagName: string): string {
  return tagName.trim().replace(/^v/i, '')
}

function parseSha256Digest(digest?: string | null): string | undefined {
  if (!digest) {
    return undefined
  }

  const [algorithm, value] = digest.split(':', 2)
  if (algorithm?.toLowerCase() !== 'sha256' || !value) {
    return undefined
  }

  return value
}

function normalizeRelease(release: GithubRelease): UpdateVersion | null {
  if (release.draft || release.prerelease || !release.published_at) {
    return null
  }

  const version = normalizeVersion(release.tag_name)
  if (!version) {
    return null
  }

  const files = release.assets
    .filter((asset) => asset.state === 'uploaded' && asset.browser_download_url)
    .map<UpdateFile>((asset) => ({
      name: asset.name,
      size: asset.size,
      sha256: parseSha256Digest(asset.digest),
      url: asset.browser_download_url
    }))

  return {
    version,
    tagName: release.tag_name,
    pub_time: release.published_at,
    description: release.body ?? undefined,
    releaseUrl: release.html_url,
    files
  }
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
    let response: Awaited<ReturnType<typeof fetch>>
    try {
      response = await fetch(GITHUB_RELEASES_API_URL, {
        method: 'GET',
        connectTimeout: 15000,
        headers: {
          Accept: 'application/vnd.github+json',
          'X-GitHub-Api-Version': '2022-11-28'
        }
      })
    } catch (error) {
      throw new Error(`Failed to fetch releases from GitHub: ${String(error)}`)
    }

    if (response.status !== 200) {
      throw new Error(
        `Failed to fetch releases from GitHub: ${response.status} ${response.statusText}`
      )
    }

    const releases = (await response.json()) as GithubRelease[]

    return {
      versions: releases
        .map(normalizeRelease)
        .filter((release): release is UpdateVersion => release !== null)
    }
  }
}
