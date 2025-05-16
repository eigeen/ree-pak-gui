import { getExePath } from '@/api/tauri/utils'
import { join } from '@tauri-apps/api/path'

const DATA_PATH = 'ree-pak-tools'

export function parentPath(path: string): string {
  // '/' or '\'
  const lastSlashIndex = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
  return path.substring(0, lastSlashIndex)
}

export async function getDataDir(): Promise<string> {
  const exePath = await getExePath()
  const exeDir = parentPath(exePath)
  return await join(exeDir, DATA_PATH)
}
