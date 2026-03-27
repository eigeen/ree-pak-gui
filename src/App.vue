<template>
  <div class="app-shell">
    <div class="relative z-10">
      <Menu />
      <main class="app-container pb-8 pt-6">
        <RouterView />
      </main>
    </div>
    <Settings />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView } from 'vue-router'
import { useSettingsStore } from '@/store/settings'
import { useWorkStore } from '@/store/work'
import { ShowError, ShowWarn } from '@/utils/message'

const settingsStore = useSettingsStore()
const workStore = useWorkStore()

onMounted(async () => {
  try {
    if (!settingsStore.settings) {
      await settingsStore.loadSettings()
    }
  } catch (error) {
    ShowError(`Failed to load settings: ${error}`)
    ShowWarn('Will use default settings')
  }

  try {
    await workStore.loadWorkRecords()
  } catch (error) {
    console.error('Failed to load work records:', error)
  }
})
</script>
