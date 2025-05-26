import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface FileListInfo {
  file_name: string
  game: string
  platform: string
  tags: string[]
  description: string
}

export const useFilelistStore = defineStore('filelist', () => {
  const localList = ref<FileListInfo[]>([])
  // Additional file paths, edited by user.
  const additionalList = ref<string[]>([])
  const onlineList = ref<FileListInfo[]>([])

  return {
    localList,
    additionalList,
    onlineList,
  }
})
