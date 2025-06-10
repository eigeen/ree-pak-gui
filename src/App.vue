<template>
  <v-app>
    <v-main class="app-root">
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
import { ShowError, ShowWarn } from '@/utils/message'

const settingsStore = useSettingsStore()

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
})
</script>

<style lang="scss">
html, body {
  /* disable vuetify scrollbar */
  overflow-y: auto !important;
  background-color: #f5f5f5;
}

html::-webkit-scrollbar {
  display: none;
}

.el-menu--horizontal {
  --el-menu-horizontal-height: 50px;
}

.app-root {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
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
