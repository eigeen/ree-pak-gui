import { createApp } from 'vue'
import App from './App.vue'

import 'unfonts.css'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './styles/global.css'

// Plugins
import { registerPlugins } from '@/plugins'

const app = createApp(App)

document.documentElement.classList.add('dark')
document.documentElement.dataset.theme = 'dark'

registerPlugins(app)

app.mount('#app')
