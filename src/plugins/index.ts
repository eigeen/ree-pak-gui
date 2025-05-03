import vuetify from './vuetify';
import router from '@/router'
import system from './system'
import { createPinia } from 'pinia';

// Types
import type { App } from 'vue'

export function registerPlugins (app: App) {
  app
    .use(vuetify)
    .use(router)
    .use(system)
    .use(createPinia())
}
