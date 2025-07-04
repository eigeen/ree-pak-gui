import { createApp } from 'vue'
import App from './App.vue'

import 'unfonts.css'
import 'element-plus/dist/index.css'
import './styles/global.css'

// Plugins
import { registerPlugins } from '@/plugins'

const app = createApp(App)

registerPlugins(app)

app.mount('#app')
