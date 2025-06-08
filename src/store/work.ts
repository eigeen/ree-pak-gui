/**
 * @file work.ts
 * Open project work data store.
 */

import { getLocalDir } from '@/lib/localDir'
import { join } from '@tauri-apps/api/path'
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'

interface WorkRecord {
  unpack: UnpackWork
}

interface UnpackWork {
  fileList: string
  paks: string[]
}

const FILE_NAME = 'work.json'

export const useWorkStore = defineStore('work', () => {
  const unpack = ref<UnpackWork>({
    fileList: '',
    paks: []
  })

  const workRecord = computed(() => {
    return {
      unpack: unpack.value
    }
  })

  async function loadWorkRecords() {
    const dataDir = await getLocalDir()
    const workFile = await join(dataDir, FILE_NAME)
    console.log(`Loading work records from ${workFile}`)
    const content = await readTextFile(workFile)
    const work = JSON.parse(content)
    if (work.unpack) {
      unpack.value = work.unpack
    }
  }

  async function saveFile(data: WorkRecord, fileName: string) {
    const dataDir = await getLocalDir()
    const filePath = await join(dataDir, fileName)
    const content = JSON.stringify(data)
    await writeTextFile(filePath, content)
    console.log('Saved work records to file')
  }

  watch(
    workRecord,
    async () => {
      await saveFile(workRecord.value, FILE_NAME)
    },
    { deep: true }
  )

  return { unpack, loadWorkRecords, saveFile }
})
