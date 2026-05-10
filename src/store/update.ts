import type { AppUpdateInfo } from '@/api/tauri/update'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUpdateStore = defineStore('update', () => {
  const hasChecked = ref(false)
  const updateVersion = ref<AppUpdateInfo | null>(null)

  return {
    hasChecked,
    updateVersion
  }
})
