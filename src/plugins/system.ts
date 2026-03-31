import { ElMessage } from 'element-plus'
import { listen } from '@tauri-apps/api/event'
import type { App } from 'vue'
import { useSystemLogStore } from '@/store/system'

type LogLevel = 'error' | 'warn' | 'info' | 'debug'

type SystemEvent = {
  event: 'log'
  data: {
    level: LogLevel
    message: string
  }
}

export default {
  install: (_app: App) => {
    const systemLogStore = useSystemLogStore()

    listen<SystemEvent>('system', (event) => {
      const eventType = event.payload.event
      const data = event.payload.data
      if (eventType === 'log') {
        systemLogStore.append(data.level, data.message)

        if (data.level === 'error') {
          ElMessage({
            showClose: true,
            customClass: 'app-toast app-toast-path',
            message: data.message,
            type: 'error',
            duration: 5000
          })
        }
      }
    })
  }
}
