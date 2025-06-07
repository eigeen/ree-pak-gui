import { UpdateAPI } from '@/api/http/update'
import type { UpdateMetadata, UpdateVersion, UpdateFile } from '@/api/http/update'
import { getCompileInfo, performUpdate, zipExtractFile, type CompileInfo } from '@/api/tauri/utils'
import { fetchWithSpeedCheck } from '@/lib/http/download'
import { getTempDir } from '@/lib/localDir'

import { join } from '@tauri-apps/api/path'
import { exists, writeFile } from '@tauri-apps/plugin-fs'
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
    if (this.compileInfo.version === '') {
      await this.initialize()
    }

    // 获取最新的更新元数据
    this.updateMetadata = await this.updateApi.fetchUpdateMetadata()
    console.info('Fetched update metadata:', this.updateMetadata)
    // 先检查version，如果为最新版本，再检查pub_time
    const validVersions = this.updateMetadata.versions.filter((v) => v.channel === 'release')
    if (!validVersions) {
      return null
    }

    // 获取当前最新版本信息
    const latestVerInfo = validVersions.reduce((acc: UpdateVersion | null, cur: UpdateVersion) => {
      if (!acc) {
        return cur
      }
      if (semver.lt(cur.version, acc.version)) {
        return cur
      }
      return acc
    }, null)
    if (!latestVerInfo) {
      console.info('No available updates')
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

    if (currVersion > latestVersion) {
      console.info('Current version is same or newer than latest version')
      return null
    }
    // 如果版本一致，进行发布时间对比
    if (currVersion === latestVersion) {
      // 发布时间检查
      const currCommitTime = new Date(this.compileInfo.commitTime)
      const latestPubTime = new Date(latestVerInfo.pub_time)
      if (currCommitTime < latestPubTime) {
        console.info(
          `New version available: ${latestVerInfo.version} (publish time newer: ${latestPubTime})`
        )
        this.targetVersion = latestVerInfo
        return latestVerInfo
      }
      return null
    }
    if (currVersion < latestVersion) {
      console.info(`New version available: ${latestVerInfo.version}`)
      this.targetVersion = latestVerInfo
      return latestVerInfo
    }

    console.warn('Unreachable')
    return null
  }

  /**
   * Download update in the target directory, and wait for performation.
   */
  public async downloadUpdate(onEvent?: (event: any) => Promise<void>) {
    if (!this.targetVersion) {
      throw new Error('Internal error: No target version available. Check for updates first.')
    }
    console.log(
      `Starting update to ${this.targetVersion.channel} version ${this.targetVersion.version} (${this.targetVersion.pub_time})`
    )

    if (this.updateMetadata === null) {
      throw new Error('Update metadata not available. Fetch update metadata first.')
    }

    // 下载并校验文件
    let { platform, arch } = await getCompileInfo()
    let targetFile: UpdateFile | undefined = undefined
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

    // 下载文件
    let lastError: any = null
    for (const url of targetFile.urls) {
      try {
        // download file
        const blob = await fetchWithSpeedCheck(url, {}, onEvent)

        // 检查Hash
        const arrayBuffer = await blob.arrayBuffer()
        const hash = await crypto.subtle.digest('SHA-256', arrayBuffer)
        const downloadedSha256 = Array.from(new Uint8Array(hash), (byte) =>
          byte.toString(16).padStart(2, '0')
        ).join('')
        if (downloadedSha256 !== targetFile.sha256) {
          throw new Error('Checksum mismatch. Download failed.')
        }

        // 写入指定目录
        const tempDir = await getTempDir(true)
        const downloadPath = await join(tempDir, targetFile.name)
        await writeFile(downloadPath, new Uint8Array(arrayBuffer))

        // 检查是否需要解压
        if (targetFile.name.endsWith('.zip')) {
          await zipExtractFile(downloadPath, tempDir)
          // 检查是否正确输出文件
          // TODO: supports non-exe extension
          const extractedPath = await join(tempDir, targetFile.name.replace('.zip', '.exe'))
          if (!(await exists(extractedPath))) {
            throw new Error(
              `Extracted file not found after zip extraction: expected ${extractedPath}`
            )
          }
          this.updateFilePath = extractedPath
        } else {
          this.updateFilePath = downloadPath
        }
        break
      } catch (err: any) {
        lastError = err
      }
    }

    if (lastError) {
      throw new Error(`Failed to download update file from all sources: ${lastError}`)
    }
  }

  /**
   * Apply update and relaunch the app.
   */
  public async performUpdate() {
    if (!this.updateFilePath) {
      throw new Error('Update file path not set. Download update first.')
    }

    await performUpdate(this.updateFilePath)
    await relaunch()
  }
}
