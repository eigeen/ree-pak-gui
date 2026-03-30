<template>
  <div class="app-shell">
    <div class="desktop-shell">
      <Menu />
      <main class="desktop-main">
        <DesktopTabPanels
          v-if="activeTopLevelTab"
          :active-value="activeTopLevelTab"
          :items="topLevelTabPanels"
        />
        <RouterView v-else />
      </main>
    </div>
    <Settings />
    <TaskProgressPanel />
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, unref, type Ref } from 'vue'
import { RouterView } from 'vue-router'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import DesktopTabPanels, { type DesktopTabPanelItem } from '@/components/DesktopTabPanels.vue'
import { setAppLocale } from '@/plugins/i18n'
import { useSettingsStore, type AppSettings } from '@/store/settings'
import { useWorkStore } from '@/store/work'
import Settings from '@/components/Settings/Settings.vue'
import TaskProgressPanel from '@/components/TaskProgressPanel.vue'
import { ShowError, ShowWarn } from '@/utils/message'

const settingsStore = useSettingsStore()
const workStore = useWorkStore()
const route = useRoute()
const { t } = useI18n()
const settings = computed(() => unref(settingsStore.settings as unknown as Ref<AppSettings>))

const topLevelTabPanels: DesktopTabPanelItem[] = [
  {
    value: 'unpack',
    component: defineAsyncComponent(() => import('@/pages/UnpackView.vue')),
    unmountInactive: true
  },
  {
    value: 'repack',
    component: defineAsyncComponent(() => import('@/pages/RepackView.vue')),
    unmountInactive: true
  },
  {
    value: 'settings',
    component: defineAsyncComponent(() => import('@/pages/SettingsView.vue')),
    unmountInactive: true
  }
]

const activeTopLevelTab = computed(() => {
  if (route.name === 'UnpackView') {
    return 'unpack'
  }

  if (route.name === 'RepackView') {
    return 'repack'
  }

  if (route.name === 'SettingsView') {
    return 'settings'
  }

  return ''
})

onMounted(async () => {
  try {
    await settingsStore.loadSettings()
    setAppLocale(settings.value.language)
  } catch (error) {
    ShowError(t('global.failedLoadSettings', { error: String(error) }))
    ShowWarn(t('global.useDefaultSettings'))
  }

  try {
    await workStore.loadWorkRecords()
  } catch (error) {
    console.error('Failed to load work records:', error)
  }
})
</script>
