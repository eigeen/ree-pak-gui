import { invoke } from '@tauri-apps/api/core'

export interface FileListInfo {
    name: string,
    absPath: string,
}

export function file_table_get_list(): Promise<FileListInfo[]> {
    return invoke('file_table_get_list')
}

export function file_table_load(path: string): Promise<void> {
    return invoke('file_table_load', { path })
}
