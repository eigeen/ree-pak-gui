import { computed, watch } from 'vue'
import { usePreferredDark } from '@vueuse/core'
import { useSettingsStore, type AppSettings, type ThemeMode } from '@/store/settings'

let hasThemeSync = false

function resolveSettingsState(settings: unknown): AppSettings | undefined {
  if (!settings || typeof settings !== 'object') {
    return undefined
  }

  if ('value' in settings) {
    return (settings as { value?: AppSettings }).value
  }

  return settings as AppSettings
}

export function useAppTheme() {
  const settingsStore = useSettingsStore()
  const preferredDark = usePreferredDark()

  const themeMode = computed<ThemeMode>({
    get: () => resolveSettingsState(settingsStore.settings)?.theme ?? 'system',
    set: (value) => {
      const settings = resolveSettingsState(settingsStore.settings)
      if (!settings) {
        return
      }

      settings.theme = value
    }
  })

  const isDark = computed(() => {
    if (themeMode.value === 'dark') {
      return true
    }

    if (themeMode.value === 'light') {
      return false
    }

    return preferredDark.value
  })

  if (!hasThemeSync) {
    hasThemeSync = true
    watch(
      isDark,
      (value) => {
        document.documentElement.classList.toggle('dark', value)
        document.documentElement.dataset.theme = value ? 'dark' : 'light'
      },
      { immediate: true }
    )
  }

  function toggleTheme() {
    themeMode.value = isDark.value ? 'light' : 'dark'
  }

  return {
    themeMode,
    isDark,
    toggleTheme
  }
}
