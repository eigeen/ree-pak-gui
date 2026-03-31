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
  type PakEntry,
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
    finishFileCount: 0
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
      finishFileCount: 0
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
    // 创建新的对象引用以确保响应式更新
    this.progress = { ...this.progress, ...updates }
    this.updateCallbacks()
  }

  private updateResult(updates: Partial<ExportResult>): void {
    // 创建新的对象引用以确保响应式更新
    this.result = { ...this.result, ...updates }
    this.updateCallbacks()
  }

  async handleExport(inputFiles: FileItem[], exportConfig: ExportConfig): Promise<void> {
    logFrontendInfo(
      'repack.export',
      `start inputs=${inputFiles.length} mode=${exportConfig.mode} auto_detect_root=${exportConfig.autoDetectRoot} fast_mode=${exportConfig.fastMode}`
    )

    this.resetExport()

    try {
      if (exportConfig.mode === 'individual') {
        await this.handleIndividualExport(inputFiles, exportConfig)
      } else if (exportConfig.mode === 'single') {
        await this.handleMergeExport(inputFiles, exportConfig)
      }
    } catch (e) {
      logFrontendError('repack.export', 'export failed', e)
      ShowError(e)
    }
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

    const outputFiles: string[] = []

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

        const processedSources = await this.processSources([file], exportConfig, {})
        logFrontendDebug(
          'repack.process-sources',
          `individual output=${outputPath} sources=${processedSources.length}`
        )

        await pak_pack(processedSources, outputPath, channel)
        outputFiles.push(outputPath)

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

      const conflicts = await this.analyzeConflicts(inputFiles)

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
      const processedSources = await this.processSources(
        inputFiles,
        exportConfig,
        this.conflictResolutions
      )
      logFrontendDebug(
        'repack.process-sources',
        `merge output=${outputPath} sources=${processedSources.length}`
      )

      const channel = new Channel<PackProgressEvent>()
      channel.onmessage = (event) => {
        this.handlePackProgress(event)
      }

      await pak_pack(processedSources, outputPath, channel)

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

  async analyzeConflicts(files: FileItem[]): Promise<ConflictFile[]> {
    try {
      logFrontendInfo('repack.conflicts', `scan inputs=${files.length}`)
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
            fileMap.get(key)?.push({
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
            fileMap.get(key)?.push({
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
      logFrontendError('repack.conflicts', 'analyze conflicts failed', error)
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
        logFrontendError('repack.scan-folder', `scan failed path=${currentPath}`, error)
      }
    }

    await scanRecursive(folderPath, folderPath)
    return files
  }

  private async processSources(
    files: FileItem[],
    exportConfig: ExportConfig,
    _resolutions: { [relativePath: string]: number }
  ): Promise<string[]> {
    const sourcePaths = files.map((f) => f.path)

    if (exportConfig.autoDetectRoot) {
      const processedPaths: string[] = []

      for (const path of sourcePaths) {
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

  setConflictResolutions(resolutions: { [relativePath: string]: number }): void {
    this.conflictResolutions = resolutions
  }
}
