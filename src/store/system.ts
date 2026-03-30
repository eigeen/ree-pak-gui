import { defineStore } from 'pinia'
import { ref } from 'vue'

export type SystemLogLevel = 'error' | 'warn' | 'info' | 'debug'

export interface SystemLogEntry {
  id: number
  level: SystemLogLevel
  message: string
  createdAt: string
}

const MAX_LOG_ENTRIES = 300
const BLOCKED_MESSAGE_KEYWORDS = ['emitted without']

export const useSystemLogStore = defineStore('system-log', () => {
  const entries = ref<SystemLogEntry[]>([])
  const nextId = ref(0)

  function append(level: SystemLogLevel, message: string) {
    const normalizedMessage = message.toLowerCase()
    if (BLOCKED_MESSAGE_KEYWORDS.some((keyword) => normalizedMessage.includes(keyword))) {
      return
    }

    const entry: SystemLogEntry = {
      id: nextId.value++,
      level,
      message,
      createdAt: new Date().toISOString()
    }

    entries.value = [...entries.value.slice(-(MAX_LOG_ENTRIES - 1)), entry]
  }

  return {
    entries,
    append
  }
})
