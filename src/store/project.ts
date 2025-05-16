import { getDataDir } from '@/utils/path'
import { join } from '@tauri-apps/api/path'
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

type UnpackProject = {
  fileList: string
  paks: string[]
}

const UNPACK_FILE_NAME = 'unpack.project.json'

export const useProjectStore = defineStore('project', () => {
  const unpack = ref<UnpackProject>({
    fileList: '',
    paks: []
  })

  async function loadProjects() {
    const dataDir = await getDataDir()
    const projectFile = await join(dataDir, UNPACK_FILE_NAME)
    console.log(`Loading unpack.project from ${projectFile}`)
    const content = await readTextFile(projectFile)
    const project = JSON.parse(content)
    unpack.value = project
  }

  async function saveFile(data: UnpackProject, fileName: string) {
    const dataDir = await getDataDir()
    const projectFile = await join(dataDir, fileName)
    const content = JSON.stringify(data)
    await writeTextFile(projectFile, content)
  }

  watch(
    unpack,
    async () => {
      await saveFile(unpack.value, UNPACK_FILE_NAME)
    },
    { deep: true }
  )

  return { unpack, loadProjects }
})
