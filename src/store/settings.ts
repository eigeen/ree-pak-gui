import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { exists, mkdir, readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { join } from '@tauri-apps/api/path'
import { getParentPath } from '@/utils/path'
import { getLocalDir } from '@/lib/localDir'

const SETTINGS_FILE_NAME = 'settings.json'

type Settings = {
  version: string
  language?: string
  preview: {
    showTexturePreview: boolean
  }
  unpack: {
    extractAbsolutePath: boolean
  }
}

const defaultSettings: Settings = {
  version: '1',
  preview: {
    showTexturePreview: true
  },
  unpack: {
    extractAbsolutePath: false
  }
}

function normalizeSettings(raw: Partial<Settings> | null | undefined): Settings {
  const legacyExtractFullPath = (raw?.unpack as { extractFullPath?: boolean } | undefined)
    ?.extractFullPath

  return {
    version: raw?.version ?? defaultSettings.version,
    language: raw?.language,
    preview: {
      showTexturePreview:
        raw?.preview?.showTexturePreview ?? defaultSettings.preview.showTexturePreview
    },
    unpack: {
      extractAbsolutePath:
        raw?.unpack?.extractAbsolutePath ??
        legacyExtractFullPath ??
        defaultSettings.unpack.extractAbsolutePath
    }
  }
}

export const useSettingsStore = defineStore('settings', () => {
  const showSettings = ref(false)
  const autoSave = ref(true)
  const settings = ref<Settings>(defaultSettings)

  async function getSettingsPath(): Promise<string> {
    const dataDir = await getLocalDir()
    return await join(dataDir, SETTINGS_FILE_NAME)
  }

  const loadSettings = async () => {
    const settingsPath = await getSettingsPath()
    console.log(`Loading settings from ${settingsPath}`)
    // if not exists, create default settings
    if (!(await exists(settingsPath))) {
      settings.value = defaultSettings
      await saveSettings()
      return
    }

    const settingsContent = await readTextFile(settingsPath)
    const settingsJson = JSON.parse(settingsContent)
    // validation
    if (settingsJson.version !== '1') {
      throw new Error(`Invalid settings file version ${settingsJson.version}`)
    }
    settings.value = normalizeSettings(settingsJson)
  }

  const saveSettings = async (byAutoSave = false) => {
    if (!settings.value) {
      throw new Error('Settings not loaded')
    }
    // prevent manual save when auto save is enabled
    if (!byAutoSave && autoSave.value) {
      return
    }
    const settingsPath = await getSettingsPath()
    const settingsDir = getParentPath(settingsPath)
    if (!settingsDir) {
      throw new Error('Failed to get settings directory')
    }

    if (!(await exists(settingsDir))) {
      await mkdir(settingsDir, { recursive: true })
    }
    const settingsContent = JSON.stringify(settings.value)
    await writeTextFile(settingsPath, settingsContent)
  }

  // auto save settings
  watch(
    settings,
    async () => {
      if (autoSave.value) {
        await saveSettings(true)
      }
    },
    { deep: true }
  )

  return {
    showSettings,
    autoSave,
    settings,
    loadSettings,
    saveSettings
  }
})
