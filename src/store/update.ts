import type { UpdateVersion } from '@/api/http/update'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUpdateStore = defineStore('update', () => {
  const hasChecked = ref(false)
  const updateVersion = ref<UpdateVersion | null>(null)

  return {
    hasChecked,
    updateVersion
  }
})
