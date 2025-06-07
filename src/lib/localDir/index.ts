import { getExePath } from '@/api/tauri/utils'
import { parentPath } from '@/utils/path'
import { join } from '@tauri-apps/api/path'
import { exists, mkdir } from '@tauri-apps/plugin-fs'

const LOCAL_DIR_NAME = 'ree-pak-tools'

export async function getLocalDir(create: boolean = true): Promise<string> {
  const exePath = await getExePath()
  const exeDir = parentPath(exePath)
  const dirPath = await join(exeDir, LOCAL_DIR_NAME)
  if (create && !(await exists(dirPath))) {
    await mkdir(dirPath, { recursive: true })
  }
  return dirPath
}

export async function getDownloadingDir(create: boolean = true): Promise<string> {
  const localDir = await getLocalDir()
  const dirPath = await join(localDir, 'downloading')
  if (create && !(await exists(dirPath))) {
    await mkdir(dirPath, { recursive: true })
  }
  return dirPath
}

export async function getTempDir(create: boolean = true): Promise<string> {
  const localDir = await getLocalDir()
  const dirPath = await join(localDir, 'temp')
  if (create && !(await exists(dirPath))) {
    await mkdir(dirPath, { recursive: true })
  }
  return dirPath
}

export async function getFileListDir(create: boolean = true): Promise<string> {
  const localDir = await getLocalDir()
  const dirPath = await join(localDir, 'filelist')
  if (create && !(await exists(dirPath))) {
    await mkdir(dirPath, { recursive: true })
  }
  return dirPath
}
