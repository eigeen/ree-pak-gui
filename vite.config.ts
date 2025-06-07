import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'

import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import Unfonts from 'unplugin-fonts/vite'
// element-plus
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'
// vuetify
import Vuetify, { transformAssetUrls } from 'vite-plugin-vuetify'

import { fileURLToPath, URL } from 'node:url'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vueJsx(),
    AutoImport({
      resolvers: [ElementPlusResolver()]
    }),
    Components({
      resolvers: [ElementPlusResolver()]
    }),
    // https://github.com/vuetifyjs/vuetify-loader/tree/master/packages/vite-plugin#readme
    Vuetify({
      autoImport: true,
      styles: {
        configFile: 'src/styles/settings.scss'
      }
    }),
    Unfonts({
      custom: {
        families: [
          {
            name: 'MiSans',
            local: 'MiSans',
            src: './src/assets/fonts/MiSans/*.woff2'
          },
          {
            name: 'MiSans Latin',
            local: 'MiSans Latin',
            src: './src/assets/fonts/MiSans-Latin/*.ttf'
          }
        ],
        display: 'swap',
        preload: true,
        prefetch: false,
        injectTo: 'head-prepend'
      }
    })
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // Tauri expects a fixed port, fail if that port is not available
  server: {
    strictPort: true
  },
  css: {
    preprocessorOptions: {
      sass: {
        api: 'modern-compiler'
      }
    }
  },
  // to access the Tauri environment variables set by the CLI with information about the current target
  envPrefix: [
    'VITE_',
    'TAURI_PLATFORM',
    'TAURI_ARCH',
    'TAURI_FAMILY',
    'TAURI_PLATFORM_VERSION',
    'TAURI_PLATFORM_TYPE',
    'TAURI_DEBUG'
  ],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // 为调试构建生成源代码映射 (sourcemap)
    sourcemap: !!process.env.TAURI_DEBUG
  }
})
