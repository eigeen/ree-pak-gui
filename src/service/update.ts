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

const CHANNEL = 'release'

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
        if (this.compileInfo.version === '') {
          await this.initialize()
        }

        // 获取最新的更新元数据
        this.updateMetadata = await this.updateApi.fetchUpdateMetadata()
        logFrontendDebug(
          'update.check',
          `metadata fetched versions=${this.updateMetadata.versions.length}`
        )

        // 先检查version，如果为最新版本，再检查pub_time
        const validVersions = this.updateMetadata.versions.filter((v) => v.channel === CHANNEL)
        if (!validVersions) {
          return null
        }

        // 获取当前最新版本信息
        const latestVerInfo = validVersions.reduce(
          (acc: UpdateVersion | null, cur: UpdateVersion) => {
            if (!acc) {
              return cur
            }
            if (semver.gt(cur.version, acc.version)) {
              return cur
            }
            return acc
          },
          null
        )
        if (!latestVerInfo) {
          return null
        }

        const latestSemver = semver.valid(latestVerInfo.version)
        if (!latestSemver) {
          throw new Error(`Invalid version from remote: ${latestVerInfo.version}`)
        }

        // 版本对比
        const toSemVer = (version: string) => {
          const parsed = semver.parse(version)
          if (!parsed) {
            throw new Error(`Invalid version: ${version}`)
          }
          return parsed
        }

        const currVersion = toSemVer(this.compileInfo.version)
        const latestVersion = toSemVer(latestSemver)

        if (semver.gt(currVersion, latestVersion)) {
          return null
        }
        // 如果版本一致，进行发布时间对比
        if (semver.eq(currVersion, latestVersion)) {
          // 发布时间检查
          const currCommitTime = new Date(this.compileInfo.commitTime)
          const latestPubTime = new Date(latestVerInfo.pub_time)
          if (currCommitTime < latestPubTime) {
            this.targetVersion = latestVerInfo
            return latestVerInfo
          }
          return null
        }
        if (semver.lt(currVersion, latestVersion)) {
          this.targetVersion = latestVerInfo
          return latestVerInfo
        }

        return null
      },
      {
        start: `check channel=${CHANNEL}`,
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
          // 检查Hash
          const array = await readFile(downloadPath)
          const downloadedSha256 = await sha256Hex(array)
          if (downloadedSha256 === targetFile.sha256) {
            newArchivePath = downloadPath
            reusedCache = true
          }
        }

        // 如果文件不存在则下载文件
        let lastError: unknown = null
        if (!newArchivePath) {
          for (const url of targetFile.urls) {
            try {
              logFrontendDebug('update.download', `try url=${url}`)
              const blob = await fetchWithSpeedCheck(url, {}, onEvent)

              // 检查Hash
              const arrayBuffer = await blob.arrayBuffer()
              const downloadedSha256 = await sha256Hex(arrayBuffer)
              if (downloadedSha256 !== targetFile.sha256) {
                throw new Error('Checksum mismatch. Download failed.')
              }

              // 写入指定目录
              await writeFile(downloadPath, new Uint8Array(arrayBuffer))
              newArchivePath = downloadPath
              lastError = null
              break
            } catch (error) {
              lastError = error
            }
          }
        }
        if (lastError || !newArchivePath) {
          throw new Error(`Failed to download update file from all sources: ${lastError}`)
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
