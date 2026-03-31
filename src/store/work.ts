/**
 * @file work.ts
 * Open project work data store.
 */

import { getLocalDir } from '@/lib/localDir'
import { logFrontendDebug, logFrontendError, runLoggedTask } from '@/utils/frontendLog'
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
  explorerLayoutMode: 'tile' | 'details'
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
    filterUseRegex: false,
    explorerLayoutMode: 'details'
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
    await runLoggedTask(
      'workspace.load',
      async () => {
        const dataDir = await getLocalDir()
        const workFile = await join(dataDir, FILE_NAME)
        const content = await readTextFile(workFile)
        const work = JSON.parse(content)

        if (work.unpack) {
          unpack.value = {
            fileList: '',
            paks: [],
            filterText: '',
            filterUseRegex: false,
            explorerLayoutMode: 'details',
            ...work.unpack
          }
        }
        if (work.pack) {
          pack.value = work.pack
        }

        return {
          workFile,
          unpackPakCount: Array.isArray(work.unpack?.paks) ? work.unpack.paks.length : 0,
          packInputCount: Array.isArray(work.pack?.inputFiles) ? work.pack.inputFiles.length : 0
        }
      },
      {
        start: `load file=${FILE_NAME}`,
        success: ({ workFile, unpackPakCount, packInputCount }) =>
          `loaded file=${workFile} unpack_paks=${unpackPakCount} pack_inputs=${packInputCount}`
      }
    )
  }

  async function saveFile(data: WorkRecord, fileName: string) {
    try {
      const dataDir = await getLocalDir()
      const filePath = await join(dataDir, fileName)
      const content = JSON.stringify(data)
      await writeTextFile(filePath, content)
      logFrontendDebug('workspace.save', `saved file=${filePath}`)
    } catch (error) {
      logFrontendError('workspace.save', `save failed file=${fileName}`, error)
      throw error
    }
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
