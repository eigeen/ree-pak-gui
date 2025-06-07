import { getLocalDir } from '@/lib/localDir'
import { join } from '@tauri-apps/api/path'
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'

interface RecentRecord {
  unpack: UnpackHistory
}

interface UnpackHistory {
  fileList: string
  paks: string[]
}

const FILE_NAME = 'recent.json'

export const useRecentStore = defineStore('recent', () => {
  const unpack = ref<UnpackHistory>({
    fileList: '',
    paks: []
  })

  const recentRecord = computed(() => {
    return {
      unpack: unpack.value
    }
  })

  async function loadRecentRecords() {
    const dataDir = await getLocalDir()
    const recentFile = await join(dataDir, FILE_NAME)
    console.log(`Loading recent records from ${recentFile}`)
    const content = await readTextFile(recentFile)
    const recent = JSON.parse(content)
    if (recent.unpack) {
      unpack.value = recent.unpack
    }
  }

  async function saveFile(data: RecentRecord, fileName: string) {
    const dataDir = await getLocalDir()
    const projectFile = await join(dataDir, fileName)
    const content = JSON.stringify(data)
    await writeTextFile(projectFile, content)
  }

  watch(
    recentRecord,
    async () => {
      await saveFile(recentRecord.value, FILE_NAME)
    },
    { deep: true }
  )

  return { unpack, loadProjects: loadRecentRecords }
})
