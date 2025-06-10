import { open } from '@tauri-apps/plugin-fs'
import { BufReader } from './bufread'
import { FileHandleReader } from './read'

export interface NameListMetadata {
  title?: string
  tags?: string[]
  update_time?: string
  [key: string]: any
}

export type SourceType = 'local' | 'remote'

export interface FileListSource {
  identifier: string
  sourceType: SourceType
  filePath: string
}

export class NameListFile {
  public source: FileListSource
  public metadata: NameListMetadata = {}
  private metadataLineCount = 0

  public constructor(identifier: string, sourceType: SourceType, filePath: string) {
    this.source = { identifier, sourceType, filePath }
  }

  /**
   * 加载并解析文件元数据（使用BufReader优化版）
   */
  public async loadMetadata(): Promise<void> {
    const file = await open(this.source.filePath, { read: true })
    try {
      this.metadata = {}
      this.metadataLineCount = 0

      const reader = new BufReader(new FileHandleReader(file))

      for await (const line of reader.lines()) {
        if (!line.startsWith('#!')) {
          break
        }

        const metaLine = line.substring(2).trim()
        const separatorIndex = metaLine.indexOf(':')
        if (separatorIndex === -1) continue

        const key = metaLine.substring(0, separatorIndex).trim()
        const value = metaLine.substring(separatorIndex + 1).trim()

        if (key.startsWith('@')) {
          const cleanKey = key.substring(1)
          this.metadata[cleanKey] =
            cleanKey === 'tags' ? value.split(',').map((tag) => tag.trim()) : value
        }
        this.metadataLineCount++
      }
    } finally {
      await file.close()
    }
  }

  /**
   * 获取元数据字段值
   */
  public getMetadata<T>(key: string, defaultValue: T): T
  public getMetadata<T>(key: string): T | undefined
  public getMetadata<T>(key: string, defaultValue?: T): T | undefined {
    const value = this.metadata[key] as T | undefined
    if (defaultValue !== undefined) {
      return value !== undefined ? value : defaultValue
    }
    return value
  }

  /**
   * 设置元数据字段值
   */
  public setMetadata(key: string, value: any) {
    this.metadata[key] = value
  }

  /**
   * 保存元数据到文件（使用文件句柄优化版）
   */
  public async saveMetadata(): Promise<void> {
    const file = await open(this.source.filePath, { read: true, write: true })
    try {
      // 获取文件总大小
      // 跳过元数据行读取剩余内容
      let remainingText = ''
      const reader = new BufReader(new FileHandleReader(file))
      let lineCount = 0
      for await (const line of reader.lines()) {
        if (lineCount++ >= this.metadataLineCount) {
          remainingText += line + '\n'
        }
      }

      // 生成新的元数据部分
      let newMetadata = ''
      for (const [key, value] of Object.entries(this.metadata)) {
        newMetadata += `#! @${key}: ${Array.isArray(value) ? value.join(', ') : value}\n`
      }

      // 截断并重写文件
      await file.truncate(0)
      await file.seek(0, 0)
      const encoder = new TextEncoder()
      await file.write(encoder.encode(newMetadata + remainingText))
    } finally {
      await file.close()
    }
  }
}
