<template>
  <v-app>
    <v-main class="root">
      <Menu></Menu>
      <div class="content">
        <router-view />
      </div>
    </v-main>
    <Settings></Settings>
  </v-app>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import { useSettingsStore } from '@/store/settings'
import { useRecentStore } from './store/recent'
import { ShowError, ShowWarn } from '@/utils'

const settingsStore = useSettingsStore()
const projectStore = useRecentStore()

onMounted(async () => {
  // initialize settings
  try {
    if (!settingsStore.settings) {
      await settingsStore.loadSettings()
    }
  } catch (error) {
    ShowError(`Failed to load settings: ${error}`)
    ShowWarn("Will use default settings")
  }
  // load projects
  try {
    await projectStore.loadProjects()
  } catch (error) {
    // failure is ok
    console.log(`Failed to load projects: ${error}`)
  }
  // get updates
  
})
</script>

<style lang="scss">
html {
  /* disable vuetify scrollbar */
  overflow-y: auto !important;
}

html::-webkit-scrollbar {
  display: none;
}

.el-menu--horizontal {
  --el-menu-horizontal-height: 50px;
}

.root {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: #f5f5f5;
}

el-menu {
  .right-group {
    display: flex;
    align-items: center;
    margin-left: auto;
  }
}

.content {
  margin: 0 10px;
  flex: 1;
}
</style>
