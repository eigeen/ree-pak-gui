import { ElMessage } from 'element-plus'
import { listen } from '@tauri-apps/api/event'
import type { App } from 'vue'

type LogLevel = 'error' | 'warn' | 'info' | 'debug'

type SystemEvent = {
  event: 'log'
  data: {
    level: LogLevel
    message: string
  }
}

export default {
  install: (app: App) => {
    listen<SystemEvent>('system', (event) => {
      const eventType = event.payload.event
      const data = event.payload.data
      console.debug('System event:', event.payload)
      if (eventType === 'log') {
        if (data.level === 'error') {
          ElMessage({
            showClose: true,
            message: data.message,
            type: 'error',
            duration: 5000
          })
          console.error('System error:', data.message)
        }
      }
    })

    console.debug('System plugin installed')
  }
}
