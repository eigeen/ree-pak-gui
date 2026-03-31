<template>
  <header class="desktop-header">
    <div class="desktop-menubar">
      <div class="flex min-w-0 items-center gap-5">
        <div class="desktop-brand">
          <PackageOpen class="size-4" />
          <span>REE Pak Tool</span>
        </div>

        <DesktopTabs v-model="topNavValue" :tabs="topNavTabs" />
      </div>

      <div class="desktop-topbar-right">
        <Button
          v-if="updateStore.updateVersion"
          variant="outline"
          size="sm"
          class="desktop-command-button relative"
          @click="showUpdateDialog"
        >
          <Download class="size-4" />
          <span>{{ t('updateDialog.updateAvailable') }}</span>
          <span class="absolute right-2 top-1.5 size-1.5 rounded-full bg-destructive" />
        </Button>

        <LanguageSelect compact :title="t('settings.languageTitle')" />

        <Button
          variant="ghost"
          size="icon-sm"
          class="desktop-icon-button"
          :title="themeButtonTitle"
          @click="toggleTheme"
        >
          <component :is="isDark ? MoonStar : SunMedium" class="size-4" />
        </Button>

        <Button
          variant="ghost"
          size="icon-sm"
          class="desktop-icon-button"
          @click="openUrl('https://github.com/eigeen/ree-pak-rs')"
        >
          <Github class="size-4" />
        </Button>
      </div>
    </div>

    <UpdateDialog ref="updateDialog" />
  </header>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Download, Github, MoonStar, PackageOpen, SunMedium } from 'lucide-vue-next'
import DesktopTabs, { type DesktopTabItem } from '@/components/DesktopTabs.vue'
import LanguageSelect from '@/components/LanguageSelect.vue'
import UpdateDialog from '@/components/UpdateDialog.vue'
import { useUpdateStore } from '@/store/update'
import { Button } from '@/components/ui/button'
import { useAppTheme } from '@/composables/theme'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const updateStore = useUpdateStore()
const { isDark, themeMode, toggleTheme } = useAppTheme()

const updateDialog = ref<{ popup: () => void } | null>(null)
const topNavTabs = computed<DesktopTabItem[]>(() => [
  {
    value: 'unpack',
    label: t('menu.unpack'),
    to: { name: 'UnpackView' }
  },
  {
    value: 'repack',
    label: t('menu.repack'),
    to: { name: 'RepackView' }
  },
  {
    value: 'settings',
    label: t('menu.settings'),
    to: { name: 'SettingsView' }
  }
])
const topNavValue = computed({
  get() {
    if (route.name === 'RepackView') {
      return 'repack'
    }

    if (route.name === 'SettingsView') {
      return 'settings'
    }

    return 'unpack'
  },
  set(value: string) {
    const target = topNavTabs.value.find((tab) => tab.value === value)?.to

    if (target) {
      void router.push(target)
    }
  }
})

const themeModeLabelKey = computed(() => {
  switch (themeMode.value) {
    case 'light':
      return 'settings.themeModeLight'
    case 'dark':
      return 'settings.themeModeDark'
    default:
      return 'settings.themeModeSystem'
  }
})

const themeButtonTitle = computed(() =>
  t('settings.themeToggleTitle', {
    mode: t(themeModeLabelKey.value),
    current: isDark.value ? t('settings.themeModeDark') : t('settings.themeModeLight')
  })
)

function showUpdateDialog() {
  updateDialog.value?.popup()
}
</script>
