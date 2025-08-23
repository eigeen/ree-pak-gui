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
  filterText: string
  filterUseRegex: boolean
}

type PackWork = {
  exportConfig: {
    mode: 'individual' | 'single'
    autoDetectRoot: boolean
    exportDirectory: string
    fastMode: boolean
  }
  inputFiles: FileItem[]
}

export interface FileItem {
  path: string
  isFile: boolean
}

const FILE_NAME = 'workspace.json'

export const useWorkStore = defineStore('work', () => {
  const unpack = ref<UnpackWork>({
    fileList: '',
    paks: [],
    filterText: '',
    filterUseRegex: false
  })

  const pack = ref<PackWork>({
    exportConfig: {
      mode: 'individual',
      autoDetectRoot: true,
      exportDirectory: '',
      fastMode: false
    },
    inputFiles: []
  })

  const workRecord = computed(() => {
    return {
      unpack: unpack.value,
      pack: pack.value
    }
  })

  async function loadWorkRecords() {
    const dataDir = await getLocalDir()
    const workFile = await join(dataDir, FILE_NAME)
    console.log(`Loading work records from ${workFile}`)
    const content = await readTextFile(workFile)
    const work = JSON.parse(content)
    console.debug('work record', work)

    if (work.unpack) {
      unpack.value = work.unpack
    }
    if (work.pack) {
      pack.value = work.pack
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

  return { unpack, pack, loadWorkRecords, saveFile }
})
