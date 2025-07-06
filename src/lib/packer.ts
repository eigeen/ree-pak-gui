import { Channel } from '@tauri-apps/api/core'
import { exists, stat, readDir, mkdir } from '@tauri-apps/plugin-fs'
import { join } from '@tauri-apps/api/path'
import { ShowError, ShowWarn } from '@/utils/message'
import {
  pak_pack,
  pak_get_header,
  pak_terminate_pack,
  type PackProgressEvent,
  type PakHeaderInfo,
  type PakEntry
} from '@/api/tauri/pak'
import type { FileItem } from '@/store/work'
import { getParentPath } from '@/utils/path'

export interface ConflictFile {
  relativePath: string
  size?: number
  modifiedDate?: Date
  sources: Array<{ sourcePath: string }>
  selectedSource: number // -1: 移除文件, 0+: 对应源文件索引
}

export interface ExportConfig {
  mode: 'individual' | 'single'
  exportDirectory: string
  autoDetectRoot: boolean
  fastMode: boolean
}

export interface ExportResult {
  success: boolean
  files: string[]
  error: string
}

export interface PackProgress {
  working: boolean
  currentFile: string
  totalFileCount: number
  finishFileCount: number
  progressValue: number
}

export interface FolderFile {
  relativePath: string
  fullPath: string
  size: number
  modifiedDate: Date
}

export class Packer {
  private progress: PackProgress = {
    working: false,
    currentFile: '',
    totalFileCount: 0,
    finishFileCount: 0,
    progressValue: 0
  }

  private result: ExportResult = {
    success: false,
    files: [],
    error: ''
  }

  private conflictResolutions: { [relativePath: string]: number } = {}

  constructor(
    private onProgressUpdate?: (progress: PackProgress) => void,
    private onResultUpdate?: (result: ExportResult) => void
  ) {}

  getProgress(): PackProgress {
    return { ...this.progress }
  }

  getResult(): ExportResult {
    return { ...this.result }
  }

  resetExport(): void {
    this.progress = {
      working: false,
      currentFile: '',
      totalFileCount: 0,
      finishFileCount: 0,
      progressValue: 0
    }
    this.result = {
      success: false,
      files: [],
      error: ''
    }
    this.updateCallbacks()
  }

  private updateCallbacks(): void {
    this.onProgressUpdate?.(this.progress)
    this.onResultUpdate?.(this.result)
  }

  private updateProgress(updates: Partial<PackProgress>): void {
    Object.assign(this.progress, updates)
    if (this.progress.totalFileCount > 0) {
      this.progress.progressValue =
        (this.progress.finishFileCount / this.progress.totalFileCount) * 100
    }
    this.updateCallbacks()
  }

  private updateResult(updates: Partial<ExportResult>): void {
    Object.assign(this.result, updates)
    this.updateCallbacks()
  }

  async handleExport(inputFiles: FileItem[], exportConfig: ExportConfig): Promise<void> {
    console.debug('handleExport', {
      files: inputFiles,
      mode: exportConfig.mode,
      autoDetectRoot: exportConfig.autoDetectRoot,
      exportDirectory: exportConfig.exportDirectory
    })

    this.resetExport()

    try {
      if (exportConfig.mode === 'individual') {
        await this.handleIndividualExport(inputFiles, exportConfig)
      } else if (exportConfig.mode === 'single') {
        await this.handleMergeExport(inputFiles, exportConfig)
      }
    } catch (e) {
      ShowError(e)
    }
  }

  private async handleIndividualExport(
    inputFiles: FileItem[],
    exportConfig: ExportConfig
  ): Promise<void> {
    this.updateProgress({
      working: true,
      totalFileCount: inputFiles.length,
      finishFileCount: 0
    })

    const outputFiles: string[] = []

    try {
      for (const file of inputFiles) {
        const outputName = this.generateOutputName(file.path, exportConfig)
        let exportDir: string
        if (!exportConfig.exportDirectory) {
          // empty export directory, use input files' directory
          let parentPath = getParentPath(file.path)
          if (!parentPath) {
            throw new Error('Failed to get parent path from input file.')
          }
          exportDir = parentPath
        } else {
          exportDir = exportConfig.exportDirectory
        }

        const uniqueOutputName = await this.generateUniqueFileName(exportDir, outputName)
        const outputPath = await join(exportDir, uniqueOutputName)

        const channel = new Channel<PackProgressEvent>()
        channel.onmessage = (event) => {
          this.handlePackProgress(event)
        }

        const processedSources = await this.processSources([file], exportConfig, {})
        await pak_pack(processedSources, outputPath, channel)
        outputFiles.push(outputPath)
      }

      this.updateProgress({ working: false })
      this.updateResult({
        success: true,
        files: outputFiles,
        error: ''
      })
    } catch (error) {
      this.updateProgress({ working: false })
      this.updateResult({
        success: false,
        files: [],
        error: error instanceof Error ? error.message : String(error)
      })
      ShowError(error)
    }
  }

  async handleMergeExport(
    inputFiles: FileItem[],
    exportConfig: ExportConfig
  ): Promise<ConflictFile[]> {
    try {
      const conflicts = await this.analyzeConflicts(inputFiles)

      if (conflicts.length > 0) {
        return conflicts
      } else {
        await this.proceedWithMergeExport(inputFiles, exportConfig)
        return []
      }
    } catch (e) {
      ShowError(e)
      return []
    }
  }

  async proceedWithMergeExport(inputFiles: FileItem[], exportConfig: ExportConfig): Promise<void> {
    if (!exportConfig.exportDirectory) {
      throw new Error('Export directory is required for merge export')
    }

    const outputName = this.generateMergedOutputName(inputFiles, exportConfig)
    const outputPath = await join(exportConfig.exportDirectory, outputName)
    if (!(await exists(outputPath))) {
      await mkdir(exportConfig.exportDirectory, { recursive: true })
    }

    try {
      const processedSources = await this.processSources(
        inputFiles,
        exportConfig,
        this.conflictResolutions
      )

      const channel = new Channel<PackProgressEvent>()
      channel.onmessage = (event) => {
        this.handlePackProgress(event)
      }

      await pak_pack(processedSources, outputPath, channel)

      this.updateProgress({ working: false })
      this.updateResult({
        success: true,
        files: [outputPath],
        error: ''
      })
    } catch (error) {
      this.updateProgress({ working: false })
      this.updateResult({
        success: false,
        files: [],
        error: error instanceof Error ? error.message : String(error)
      })
      ShowError(error)
    }
  }

  private handlePackProgress(event: PackProgressEvent): void {
    switch (event.event) {
      case 'workStart':
        this.updateProgress({
          working: true,
          totalFileCount: event.data.count,
          finishFileCount: 0
        })
        break
      case 'fileDone':
        this.updateProgress({
          currentFile: event.data.path,
          finishFileCount: event.data.finishCount
        })
        break
      case 'workFinished':
        this.updateProgress({ working: false })
        this.updateResult({
          success: true,
          files: [],
          error: ''
        })
        break
      case 'error':
        this.updateProgress({ working: false })
        this.updateResult({
          success: false,
          files: [],
          error: event.data.error
        })
        ShowError(event.data.error)
        break
    }
  }

  async analyzeConflicts(files: FileItem[]): Promise<ConflictFile[]> {
    try {
      const fileMap = new Map<
        string,
        Array<{ sourcePath: string; size: number; modifiedDate: Date }>
      >()

      for (const file of files) {
        if (file.isFile && file.path.endsWith('.pak')) {
          const headerInfo = await pak_get_header(file.path)
          for (const pakEntry of headerInfo.entries) {
            // 构建相对路径，由于PakEntry没有relativePath属性，我们需要使用其他方式
            const key = `entry_${pakEntry.hashNameLower}_${pakEntry.hashNameUpper}`
            if (!fileMap.has(key)) {
              fileMap.set(key, [])
            }
            fileMap.get(key)!.push({
              sourcePath: `${file.path}:${key}`,
              size: pakEntry.uncompressedSize,
              modifiedDate: new Date()
            })
          }
        } else {
          const folderFiles = await this.scanFolderFiles(file.path)
          for (const folderFile of folderFiles) {
            const key = folderFile.relativePath
            if (!fileMap.has(key)) {
              fileMap.set(key, [])
            }
            fileMap.get(key)!.push({
              sourcePath: folderFile.fullPath,
              size: folderFile.size,
              modifiedDate: folderFile.modifiedDate
            })
          }
        }
      }

      const conflicts: ConflictFile[] = []
      for (const [relativePath, sources] of fileMap) {
        if (sources.length > 1) {
          conflicts.push({
            relativePath,
            size: sources[0]?.size,
            modifiedDate: sources[0]?.modifiedDate,
            sources: sources.map((s) => ({ sourcePath: s.sourcePath })),
            selectedSource: sources.length - 1
          })
        }
      }

      return conflicts
    } catch (error) {
      console.error('分析冲突失败:', error)
      return []
    }
  }

  private async scanFolderFiles(folderPath: string): Promise<FolderFile[]> {
    const files: FolderFile[] = []

    const scanRecursive = async (currentPath: string, basePath: string) => {
      try {
        const entries = await readDir(currentPath)

        for (const entry of entries) {
          const fullPath = await join(currentPath, entry.name)

          if (entry.isDirectory) {
            await scanRecursive(fullPath, basePath)
          } else {
            const fileStat = await stat(fullPath)
            const relativePath = fullPath.replace(basePath, '').replace(/\\/g, '/')

            files.push({
              relativePath: relativePath.startsWith('/') ? relativePath : '/' + relativePath,
              fullPath,
              size: fileStat.size,
              modifiedDate: fileStat.mtime ? new Date(fileStat.mtime) : new Date()
            })
          }
        }
      } catch (error) {
        console.error('扫描文件夹失败:', error)
      }
    }

    await scanRecursive(folderPath, folderPath)
    return files
  }

  private async processSources(
    files: FileItem[],
    exportConfig: ExportConfig,
    resolutions: { [relativePath: string]: number }
  ): Promise<string[]> {
    let sourcePaths = files.map((f) => f.path)

    if (exportConfig.autoDetectRoot) {
      const processedPaths: string[] = []

      for (const path of sourcePaths) {
        const parts = path.split(/[/\\]/)
        const nativesIndex = parts.findIndex((p) => p === 'natives')
        const stmIndex = parts.findIndex((p) => p === 'STM')

        if (nativesIndex >= 0 && stmIndex === nativesIndex + 1) {
          const separator = path.includes('\\') ? '\\' : '/'
          const rootPath = parts.slice(0, nativesIndex + 1).join(separator)
          processedPaths.push(rootPath)
        } else {
          processedPaths.push(path)
        }
      }

      return [...new Set(processedPaths)]
    }

    return sourcePaths
  }

  private async generateUniqueFileName(directory: string, baseName: string): Promise<string> {
    let fileName = baseName
    let counter = 1

    while (await exists(await join(directory, fileName))) {
      const nameWithoutExt = baseName.replace(/\.pak$/, '')
      fileName = `${nameWithoutExt}-${counter}.pak`
      counter++
    }

    return fileName
  }

  private generateOutputName(inputPath: string, exportConfig: ExportConfig): string {
    const basename = inputPath.split(/[/\\]/).pop() || 'output'

    if (exportConfig.autoDetectRoot) {
      const parts = inputPath.split(/[/\\]/)
      const nativesIndex = parts.findIndex((p) => p === 'natives')
      if (nativesIndex > 0) {
        return `${parts[nativesIndex - 1]}.pak`
      }
    }

    return basename.endsWith('.pak') ? basename : `${basename}.pak`
  }

  private generateMergedOutputName(inputFiles: FileItem[], exportConfig: ExportConfig): string {
    if (exportConfig.autoDetectRoot && inputFiles.length > 0) {
      const firstPath = inputFiles[0].path
      const parts = firstPath.split(/[/\\]/)
      const nativesIndex = parts.findIndex((p) => p === 'natives')
      if (nativesIndex > 0) {
        return `${parts[nativesIndex - 1]}.pak`
      }
    }

    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
    return `merged-${timestamp}.pak`
  }

  async terminateExport(): Promise<void> {
    try {
      await pak_terminate_pack()
      this.resetExport()
      ShowWarn('导出操作已取消')
    } catch (error) {
      ShowError('取消导出操作失败: ' + error)
    }
  }

  setConflictResolutions(resolutions: { [relativePath: string]: number }): void {
    this.conflictResolutions = resolutions
  }
}
