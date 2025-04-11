import vuetify from './vuetify';
import router from '@/router'
import system from './system'

// Types
import type { App } from 'vue'

export function registerPlugins (app: App) {
  app
    .use(vuetify)
    .use(router)
    .use(system)
}
