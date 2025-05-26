import { fetch } from '@tauri-apps/plugin-http'

const METADATA_URLS = [
  "https://raw.githubusercontent.com/eigeen/ree-pak-gui-update/refs/heads/main/filelist_index.json",
  "https://gitee.com/eigeen/ree-pak-gui-update/raw/main/filelist_index.json"
]

interface FileListMetadata {
  servers: string[]
  files: FileListInfo[]
}

interface FileListInfo {
  game: string
  platform: string
  tags: string[]
  update_time: string
  file_name: string
  description: string
  size: number
  sha256: string
}

// const response = await fetch('https://raw.githubusercontent.com/eigeen/ree-pak-gui-update/refs/heads/main/filelist_index.json', {
//   method: 'GET'
// })
