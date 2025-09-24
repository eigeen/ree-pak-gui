import { invoke } from '@tauri-apps/api/core'

export interface FileListInfo {
  name: string
  absPath: string
}

export class FilePathList {
  static async getList(): Promise<FileListInfo[]> {
    return invoke('file_table_get_list')
  }

  static async load(path: string): Promise<void> {
    return invoke('file_table_load', { path })
  }

  static async pushPaths(filePathList: string[]): Promise<void> {
    return invoke('file_table_push_paths', { filePathList })
  }
}
