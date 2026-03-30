import type { FileListInfo } from '@/api/http/filelist'
import type { NameListFile } from '@/lib/NameListFile'
import { fileListService } from '@/service/filelist'
import { defineStore } from 'pinia'
import { ref, type Reactive } from 'vue'

export const useFileListStore = defineStore('filelist', () => {
  const localFile: { [identifier: string]: Reactive<NameListFile> } = {}
  const downloadedFile: { [identifier: string]: Reactive<NameListFile> } = {}

  const remoteManifest: { [fileName: string]: FileListInfo } = {}
  // Additional file paths, edited by user.
  const additionalList = ref<string[]>([])

  const refreshLocalSource = async () => {
    await fileListService.refreshLocalSource()
  }

  const fetchRemoteSource = async () => {
    await fileListService.fetchRemoteSource()
  }

  return {
    localFile,
    downloadedFile,
    remoteManifest,
    additionalList,
    refreshLocalSource,
    fetchRemoteSource
  }
})
