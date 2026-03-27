import router from '@/router'
import system from './system'
import { createPinia } from 'pinia'
import i18n from './i18n'

// Types
import type { App } from 'vue'

export function registerPlugins(app: App) {
  const pinia = createPinia()
  app.use(pinia as any)
  app.use(router)
  app.use(system)
  app.use(i18n)
}
