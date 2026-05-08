import { createApp } from 'vue'
import App from './App.vue'

import 'unfonts.css'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './styles/global.css'

// Plugins
import { registerPlugins } from '@/plugins'

window.addEventListener('contextmenu', (event) => {
  event.preventDefault()
})

const app = createApp(App)

registerPlugins(app)

app.mount('#app')
