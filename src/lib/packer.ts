import { Channel } from '@tauri-apps/api/core'
import { exists, readDir, mkdir } from '@tauri-apps/plugin-fs'
import { join } from '@tauri-apps/api/path'
import { ShowError } from '@/utils/message'
import {
  pak_analyze_conflicts,
  pak_pack,
  pak_terminate_pack,
  type PackConflictInfo,
  type PackConflictResolution,
  type PackOptions,
  type PackProgressEvent,
  type PackedPak
} from '@/api/tauri/pak'
import type { FileItem } from '@/store/work'
import { logFrontendDebug, logFrontendError, logFrontendInfo } from '@/utils/frontendLog'
import { getParentPath } from '@/utils/path'
import i18n from '@/plugins/i18n'

// 定义文件类型
type PackedFile = {
  path: string
  hash: [number, number]
  size: number
}

// 文件树渲染函数
export function renderFileTree(paks: PackedPak[]): string {
  if (paks.length === 0) {
    return i18n.global.t('pack.noFiles')
  }

  let result = ''

  paks.forEach((pak, pakIndex) => {
    const isLastPak = pakIndex === paks.length - 1
    const pakPrefix = isLastPak ? '└── ' : '├── '
    const pakName = pak.path.split(/[/\\]/).pop() || pak.path

    result += `${pakPrefix}📦 ${pakName} (${pak.files.length} files)\n`

    // 按路径对文件进行分组和排序
    const fileTree = buildFileTree(pak.files)
    const childPrefix = isLastPak ? '    ' : '│   '
    result += renderFileTreeNode(fileTree, childPrefix)
  })

  return result
}

// 构建文件树结构
function buildFileTree(files: PackedFile[]): FileTreeNode {
  const root: FileTreeNode = { name: '', children: new Map(), files: [] }

  files.forEach((file) => {
    const parts = file.path.split(/[/\\]/).filter((part: string) => part.length > 0)
    let current = root

    // 遍历路径的每一部分
    for (let i = 0; i < parts.length - 1; i++) {
      const part = parts[i]
      if (!part) continue

      if (!current.children.has(part)) {
        current.children.set(part, { name: part, children: new Map(), files: [] })
      }

      const next = current.children.get(part)
      if (!next) continue
      current = next
    }

    // 添加文件到最终目录
    if (parts.length > 0) {
      current.files.push(file)
    }
  })

  return root
}

// 渲染文件树节点
function renderFileTreeNode(node: FileTreeNode, prefix: string): string {
  let result = ''

  // 获取所有子目录和文件，并排序
  const children = Array.from(node.children.values()).sort((a, b) => a.name.localeCompare(b.name))
  const files = node.files.sort((a, b) => a.path.localeCompare(b.path))

  // 渲染子目录
  children.forEach((child, index) => {
    const isLast = index === children.length - 1 && files.length === 0
    const childPrefix = isLast ? '└── ' : '├── '
    const nextPrefix = prefix + (isLast ? '    ' : '│   ')

    result += `${prefix}${childPrefix}📁 ${child.name}\n`
    result += renderFileTreeNode(child, nextPrefix)
  })

  // 渲染文件
  files.forEach((file, index) => {
    const isLast = index === files.length - 1
    const filePrefix = isLast ? '└── ' : '├── '
    const fileName = file.path.split(/[/\\]/).pop() || file.path
    const fileSize = formatFileSize(file.size)

    result += `${prefix}${filePrefix}📄 ${fileName} (${fileSize})\n`
  })

  return result
}

// 格式化文件大小
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'

  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// 文件树节点接口
interface FileTreeNode {
  name: string
  children: Map<string, FileTreeNode>
  files: PackedFile[]
}

export interface ConflictFile {
  targetKey: string
  targetPath: string
  size?: number
  modifiedDate?: Date
  sources: Array<{ id: string; sourcePath: string }>
  selectedSourceId: string | null
}

export interface ExportConfig {
  mode: 'individual' | 'single'
  exportDirectory: string
  autoDetectRoot: boolean
  fastMode: boolean
  allowFileNameAsPathHash: boolean
}

export interface ExportResult {
  success: boolean
  files: PackedPak[]
  error: string
  fileTree?: string // 添加文件树字符串显示
}

export interface PackProgress {
  working: boolean
  currentFile: string
  totalFileCount: number
  finishFileCount: number
}

export class Packer {
  private progress: PackProgress = {
    working: false,
    currentFile: '',
    totalFileCount: 0,
    finishFileCount: 0
  }

  private result: ExportResult = {
    success: false,
    files: [],
    error: ''
  }

  private conflictResolutions: PackConflictResolution = {}

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
      finishFileCount: 0
    }
    this.conflictResolutions = {}
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
    // 创建新的对象引用以确保响应式更新
    this.progress = { ...this.progress, ...updates }
    this.updateCallbacks()
  }

  private updateResult(updates: Partial<ExportResult>): void {
    // 创建新的对象引用以确保响应式更新
    this.result = { ...this.result, ...updates }
    this.updateCallbacks()
  }

  async handleExport(inputFiles: FileItem[], exportConfig: ExportConfig): Promise<ConflictFile[]> {
    logFrontendInfo(
      'repack.export',
      `start inputs=${inputFiles.length} mode=${exportConfig.mode} auto_detect_root=${exportConfig.autoDetectRoot} fast_mode=${exportConfig.fastMode} filename_hash=${exportConfig.allowFileNameAsPathHash}`
    )

    this.resetExport()

    try {
      if (exportConfig.mode === 'individual') {
        await this.handleIndividualExport(inputFiles, exportConfig)
        return []
      } else if (exportConfig.mode === 'single') {
        return await this.handleMergeExport(inputFiles, exportConfig)
      }
    } catch (e) {
      logFrontendError('repack.export', 'export failed', e)
      ShowError(e)
    }

    return []
  }

  private async handleIndividualExport(
    inputFiles: FileItem[],
    exportConfig: ExportConfig
  ): Promise<void> {
    // 设置初始进度状态
    this.updateProgress({
      working: true,
      totalFileCount: inputFiles.length,
      finishFileCount: 0
    })

    function sleep(time: number) {
      return new Promise((resolve) => {
        setTimeout(resolve, time)
      })
    }

    try {
      for (const file of inputFiles) {
        const outputName = this.generateOutputName(file.path, exportConfig)
        let exportDir: string
        if (!exportConfig.exportDirectory) {
          // empty export directory, use input files' directory
          let parentPath = getParentPath(file.path)
          if (!parentPath) {
            throw new Error(i18n.global.t('pack.failedGetParentPath'))
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

        const processedSources = await this.processSources([file], exportConfig)
        logFrontendDebug(
          'repack.process-sources',
          `individual output=${outputPath} sources=${processedSources.length}`
        )

        await pak_pack(
          this.buildPackOptions(processedSources, outputPath, exportConfig, {}),
          channel
        )

        // 等待完成信号，确保文件写入完成，再开始下一个文件
        while (true) {
          await sleep(50)
          if (this.progress.finishFileCount === this.progress.totalFileCount) {
            break
          }
        }
      }

      // `pak_pack` 只负责启动后台打包线程，真实结果以后续 `workFinished` 事件为准。
    } catch (error) {
      logFrontendError('repack.export', 'individual export failed', error)
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
      // 设置初始进度状态
      this.updateProgress({
        working: true,
        totalFileCount: inputFiles.length,
        finishFileCount: 0
      })

      const conflicts = await this.analyzeConflicts(inputFiles, exportConfig)

      if (conflicts.length > 0) {
        logFrontendInfo('repack.conflicts', `detected conflicts=${conflicts.length}`)
        // 有冲突时，暂停进度状态，等待用户解决冲突
        this.updateProgress({ working: false })
        return conflicts
      } else {
        logFrontendInfo('repack.conflicts', 'no conflicts detected')
        await this.proceedWithMergeExport(inputFiles, exportConfig)
        return []
      }
    } catch (e) {
      logFrontendError('repack.conflicts', 'conflict analysis failed', e)
      this.updateProgress({ working: false })
      this.updateResult({
        success: false,
        files: [],
        error: e instanceof Error ? e.message : String(e)
      })
      ShowError(e)
      return []
    }
  }

  async proceedWithMergeExport(inputFiles: FileItem[], exportConfig: ExportConfig): Promise<void> {
    if (!exportConfig.exportDirectory) {
      throw new Error(i18n.global.t('pack.exportDirRequired'))
    }

    // 重新设置进度状态（处理冲突后重新开始）
    this.updateProgress({
      working: true,
      totalFileCount: inputFiles.length,
      finishFileCount: 0
    })

    const outputName = this.generateMergedOutputName(inputFiles, exportConfig)
    const outputPath = await join(exportConfig.exportDirectory, outputName)
    if (!(await exists(outputPath))) {
      await mkdir(exportConfig.exportDirectory, { recursive: true })
    }

    try {
      const processedSources = await this.processSources(inputFiles, exportConfig)
      logFrontendDebug(
        'repack.process-sources',
        `merge output=${outputPath} sources=${processedSources.length}`
      )

      const channel = new Channel<PackProgressEvent>()
      channel.onmessage = (event) => {
        this.handlePackProgress(event)
      }

      await pak_pack(
        this.buildPackOptions(processedSources, outputPath, exportConfig, this.conflictResolutions),
        channel
      )

      // `pak_pack` 只负责启动后台打包线程，真实结果以后续 `workFinished` 事件为准。
    } catch (error) {
      logFrontendError('repack.export', 'merge export failed', error)
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
      case 'workFinished': {
        // 任务全部完成
        this.updateProgress({ working: false, finishFileCount: this.progress.totalFileCount })
        const paks = event.data?.tree?.paks ?? []
        this.updateResult({
          success: true,
          files: paks,
          fileTree: renderFileTree(paks),
          error: ''
        })
        break
      }
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

  async analyzeConflicts(files: FileItem[], exportConfig: ExportConfig): Promise<ConflictFile[]> {
    try {
      const processedSources = await this.processSources(files, exportConfig)
      logFrontendInfo(
        'repack.conflicts',
        `scan inputs=${files.length} sources=${processedSources.length}`
      )

      const conflicts = await pak_analyze_conflicts({
        sources: processedSources,
        allowFileNameAsPathHash: exportConfig.allowFileNameAsPathHash
      })

      return conflicts.map((conflict) => this.mapConflictInfo(conflict))
    } catch (error) {
      logFrontendError('repack.conflicts', 'analyze conflicts failed', error)
      return []
    }
  }

  private async processSources(files: FileItem[], exportConfig: ExportConfig): Promise<string[]> {
    const sourcePaths = files.map((file) => file.path)

    if (exportConfig.autoDetectRoot) {
      const processedPaths: string[] = []

      for (const file of files) {
        const path = file.path
        if (file.isFile) {
          processedPaths.push(path)
          continue
        }

        // 向下查找第一个文件
        const firstFile = await this.findFirstFile(path)
        if (!firstFile) {
          processedPaths.push(path)
          continue
        }

        // check if it is a natives/STM/** path
        const parts = firstFile.split(/[/\\]/)
        const nativesIndex = parts.findIndex((p) => p.toLowerCase() === 'natives')
        const stmIndex = parts.findIndex((p) => p.toLowerCase() === 'stm')

        if (nativesIndex >= 0 && stmIndex === nativesIndex + 1) {
          const separator = firstFile.includes('\\') ? '\\' : '/'
          const rootPath = parts.slice(0, nativesIndex + 1).join(separator) // keep xxx/yyy/natives
          processedPaths.push(rootPath)
        } else {
          processedPaths.push(path)
        }
      }

      const dedupedPaths = [...new Set(processedPaths)]
      logFrontendDebug(
        'repack.process-sources',
        `auto-detected sources=${dedupedPaths.length} inputs=${sourcePaths.length}`
      )
      return dedupedPaths
    }

    logFrontendDebug(
      'repack.process-sources',
      `reuse original sources=${sourcePaths.length} auto_detect_root=false`
    )
    return sourcePaths
  }

  private async findFirstFile(rootPath: string): Promise<string> {
    const entries = await readDir(rootPath)
    for (const entry of entries) {
      if (entry.isFile) {
        return await join(rootPath, entry.name)
      } else {
        const firstFile = await this.findFirstFile(await join(rootPath, entry.name))
        if (firstFile) {
          return firstFile
        }
      }
    }
    return ''
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
      const nativesIndex = parts.indexOf('natives')
      if (nativesIndex > 0) {
        return `${parts[nativesIndex - 1]}.pak`
      }
    }

    return basename.endsWith('.pak') ? basename : `${basename}.pak`
  }

  private generateMergedOutputName(inputFiles: FileItem[], exportConfig: ExportConfig): string {
    const firstFile = inputFiles.at(0)
    if (exportConfig.autoDetectRoot && firstFile) {
      const firstPath = firstFile.path
      const parts = firstPath.split(/[/\\]/)
      const nativesIndex = parts.indexOf('natives')
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
    } catch (error) {
      ShowError(i18n.global.t('pack.failedCancelExport', { error }))
    }
  }

  setConflictResolutions(resolutions: PackConflictResolution): void {
    this.conflictResolutions = resolutions
  }

  private buildPackOptions(
    sources: string[],
    output: string,
    exportConfig: ExportConfig,
    conflictResolutions: PackConflictResolution
  ): PackOptions {
    return {
      sources,
      output,
      allowFileNameAsPathHash: exportConfig.allowFileNameAsPathHash,
      conflictResolutions
    }
  }

  private mapConflictInfo(conflict: PackConflictInfo): ConflictFile {
    return {
      targetKey: conflict.targetKey,
      targetPath: conflict.targetPath,
      size: conflict.size,
      modifiedDate:
        conflict.modifiedTimestampMs === undefined || conflict.modifiedTimestampMs === null
          ? undefined
          : new Date(conflict.modifiedTimestampMs),
      sources: conflict.sources.map((source) => ({
        id: source.id,
        sourcePath: source.sourcePath
      })),
      selectedSourceId: conflict.selectedSourceId ?? conflict.sources.at(-1)?.id ?? null
    }
  }
}
