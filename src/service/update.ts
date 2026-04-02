import { UpdateAPI } from '@/api/http/update'
import type { UpdateMetadata, UpdateVersion, UpdateFile } from '@/api/http/update'
import { getCompileInfo, performUpdate, zipExtractFile, type CompileInfo } from '@/api/tauri/utils'
import { fetchWithSpeedCheck } from '@/lib/http/download'
import { getTempDir } from '@/lib/localDir'
import { logFrontendDebug, runLoggedTask } from '@/utils/frontendLog'
import { sha256Hex } from '@/utils/hash'

import { join } from '@tauri-apps/api/path'
import { exists, readFile, writeFile } from '@tauri-apps/plugin-fs'
import { relaunch } from '@tauri-apps/plugin-process'
import semver from 'semver'

export class UpdateService {
  private static instance: UpdateService | null = null

  private updateApi: UpdateAPI
  private compileInfo: CompileInfo
  private updateMetadata: UpdateMetadata | null = null
  private targetVersion: UpdateVersion | null = null
  private updateFilePath: string | null = null

  constructor() {
    this.updateApi = UpdateAPI.getInstance()
    this.compileInfo = {
      version: '',
      commitTime: '',
      commitHash: '',
      platform: '',
      arch: ''
    }
  }

  public static getInstance(): UpdateService {
    if (!UpdateService.instance) {
      UpdateService.instance = new UpdateService()
    }
    return UpdateService.instance
  }

  public async initialize(): Promise<void> {
    this.compileInfo = await getCompileInfo()
  }

  public async checkForUpdates(): Promise<UpdateVersion | null> {
    return runLoggedTask(
      'update.check',
      async () => {
        this.targetVersion = null

        if (this.compileInfo.version === '') {
          await this.initialize()
        }

        // 获取最新的更新元数据
        this.updateMetadata = await this.updateApi.fetchUpdateMetadata()
        logFrontendDebug(
          'update.check',
          `metadata fetched versions=${this.updateMetadata.versions.length}`
        )

        if (this.updateMetadata.versions.length === 0) {
          return null
        }

        const validRemoteVersions = this.updateMetadata.versions
          .map((version) => {
            const parsed = semver.parse(version.version)
            if (!parsed) {
              logFrontendDebug('update.check', `skip invalid remote version=${version.version}`)
              return null
            }

            if (parsed.prerelease.length > 0) {
              logFrontendDebug('update.check', `skip prerelease remote version=${version.version}`)
              return null
            }

            return {
              version,
              semver: parsed
            }
          })
          .filter(
            (
              entry
            ): entry is {
              version: UpdateVersion
              semver: semver.SemVer
            } => entry !== null
          )

        if (validRemoteVersions.length === 0) {
          return null
        }

        const latestVersionEntry = validRemoteVersions.reduce((acc, cur) => {
          if (semver.gt(cur.semver, acc.semver)) {
            return cur
          }

          return acc
        })
        const latestVerInfo = latestVersionEntry.version

        const parseLocalVersion = (version: string): semver.SemVer | null => {
          const normalizedVersion = semver.valid(version) ?? semver.coerce(version)?.version
          const parsed = normalizedVersion ? semver.parse(normalizedVersion) : null

          if (!parsed) {
            logFrontendDebug('update.check', `skip invalid local version=${version}`)
            return null
          }

          return parsed
        }

        const currVersion = parseLocalVersion(this.compileInfo.version)
        const latestVersion = latestVersionEntry.semver

        if (!currVersion) {
          const currCommitTime = new Date(this.compileInfo.commitTime)
          const latestPubTime = new Date(latestVerInfo.pub_time)
          if (
            Number.isNaN(currCommitTime.getTime()) ||
            Number.isNaN(latestPubTime.getTime()) ||
            currCommitTime < latestPubTime
          ) {
            this.targetVersion = latestVerInfo
            return latestVerInfo
          }

          return null
        }

        if (semver.gt(currVersion, latestVersion)) {
          return null
        }
        if (semver.eq(currVersion, latestVersion)) {
          return null
        }
        if (semver.lt(currVersion, latestVersion)) {
          this.targetVersion = latestVerInfo
          return latestVerInfo
        }

        return null
      },
      {
        start: 'check github releases',
        success: (version) =>
          version
            ? `update available version=${version.version} published=${version.pub_time}`
            : 'no update available'
      }
    )
  }

  /**
   * Download update in the target directory, and wait for performation.
   */
  public async downloadUpdate(onEvent?: (event: any) => Promise<void>) {
    await runLoggedTask(
      'update.download',
      async () => {
        if (!this.targetVersion) {
          throw new Error('Internal error: No target version available. Check for updates first.')
        }

        if (this.updateMetadata === null) {
          throw new Error('Update metadata not available. Fetch update metadata first.')
        }

        // 准备下载信息
        const { platform, arch } = await getCompileInfo()
        let targetFile: UpdateFile | undefined
        for (const file of this.targetVersion.files) {
          if (file.name.includes(platform) && file.name.includes(arch)) {
            targetFile = file
            break
          }
        }
        if (!targetFile) {
          throw new Error(
            'No matching file found from update metadata for current platform and architecture.'
          )
        }

        let newArchivePath: string | null = null
        let reusedCache = false
        // 检查文件是否已存在下载目录
        const tempDir = await getTempDir(true)
        const downloadPath = await join(tempDir, targetFile.name)
        if (await exists(downloadPath)) {
          if (!targetFile.sha256) {
            newArchivePath = downloadPath
            reusedCache = true
          } else {
            const array = await readFile(downloadPath)
            const downloadedSha256 = await sha256Hex(array)
            if (downloadedSha256 === targetFile.sha256) {
              newArchivePath = downloadPath
              reusedCache = true
            }
          }
        }

        let lastError: unknown = null
        if (!newArchivePath) {
          try {
            logFrontendDebug('update.download', `try url=${targetFile.url}`)
            const blob = await fetchWithSpeedCheck(targetFile.url, {}, onEvent)

            const arrayBuffer = await blob.arrayBuffer()
            if (targetFile.sha256) {
              const downloadedSha256 = await sha256Hex(arrayBuffer)
              if (downloadedSha256 !== targetFile.sha256) {
                throw new Error('Checksum mismatch. Download failed.')
              }
            }

            await writeFile(downloadPath, new Uint8Array(arrayBuffer))
            newArchivePath = downloadPath
          } catch (error) {
            lastError = error
          }
        }
        if (lastError || !newArchivePath) {
          throw new Error(`Failed to download update file: ${lastError}`)
        }

        // 检查是否需要解压
        if (targetFile.name.endsWith('.zip')) {
          await zipExtractFile(newArchivePath, tempDir)
          // 检查是否正确输出文件
          const extractedPath = await join(tempDir, targetFile.name.replace('.zip', ''))
          if (!(await exists(extractedPath))) {
            throw new Error(
              `Extracted file not found after zip extraction: expected ${extractedPath}`
            )
          }
          this.updateFilePath = extractedPath
        } else {
          this.updateFilePath = newArchivePath
        }

        return {
          fileName: targetFile.name,
          reusedCache,
          extracted: targetFile.name.endsWith('.zip')
        }
      },
      {
        start: this.targetVersion
          ? `download version=${this.targetVersion.version}`
          : 'download update',
        success: ({ fileName, reusedCache, extracted }) =>
          `ready file=${fileName} cache=${reusedCache ? 'hit' : 'miss'} extracted=${extracted}`
      }
    )
  }

  /**
   * Apply update and relaunch the app.
   */
  public async performUpdate() {
    await runLoggedTask(
      'update.apply',
      async () => {
        if (!this.updateFilePath) {
          throw new Error('Update file path not set. Download update first.')
        }

        await performUpdate(this.updateFilePath)
        await relaunch()
      },
      {
        start: this.updateFilePath ? `apply file=${this.updateFilePath}` : 'apply update',
        success: 'relaunch requested'
      }
    )
  }
}
