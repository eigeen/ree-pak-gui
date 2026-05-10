import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { exists, mkdir, readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import { join } from '@tauri-apps/api/path'
import { sanitizeStoredLocale } from '@/lib/language'
import { logFrontendDebug, logFrontendError, runLoggedTask } from '@/utils/frontendLog'
import { getParentPath } from '@/utils/path'
import { getLocalDir } from '@/lib/localDir'

const SETTINGS_FILE_NAME = 'settings.json'

export type ThemeMode = 'system' | 'light' | 'dark'
export type MeshPreviewBackgroundStyle = 'dark' | 'light'
export type MeshPreviewTextureResolution = 'standard' | 'high'

export type AppSettings = {
  version: string
  language?: string
  theme?: ThemeMode
  preview: {
    showTexturePreview: boolean
    meshPreview: {
      backgroundStyle: MeshPreviewBackgroundStyle
      textureResolution: MeshPreviewTextureResolution
      showGrid: boolean
    }
  }
  unpack: {
    extractAbsolutePath: boolean
  }
}

const defaultSettings: AppSettings = {
  version: '1',
  theme: 'system',
  preview: {
    showTexturePreview: true,
    meshPreview: {
      backgroundStyle: 'dark',
      textureResolution: 'standard',
      showGrid: true
    }
  },
  unpack: {
    extractAbsolutePath: false
  }
}

function normalizeSettings(raw: Partial<AppSettings> | null | undefined): AppSettings {
  const legacyExtractFullPath = (raw?.unpack as { extractFullPath?: boolean } | undefined)
    ?.extractFullPath
  const theme = raw?.theme

  return {
    version: raw?.version ?? defaultSettings.version,
    language: sanitizeStoredLocale(raw?.language),
    theme: theme === 'light' || theme === 'dark' || theme === 'system' ? theme : 'system',
    preview: {
      showTexturePreview:
        raw?.preview?.showTexturePreview ?? defaultSettings.preview.showTexturePreview,
      meshPreview: {
        backgroundStyle:
          raw?.preview?.meshPreview?.backgroundStyle === 'light' ||
          raw?.preview?.meshPreview?.backgroundStyle === 'dark'
            ? raw.preview.meshPreview.backgroundStyle
            : defaultSettings.preview.meshPreview.backgroundStyle,
        textureResolution:
          raw?.preview?.meshPreview?.textureResolution === 'high' ||
          raw?.preview?.meshPreview?.textureResolution === 'standard'
            ? raw.preview.meshPreview.textureResolution
            : defaultSettings.preview.meshPreview.textureResolution,
        showGrid:
          raw?.preview?.meshPreview?.showGrid ?? defaultSettings.preview.meshPreview.showGrid
      }
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
  const settings = ref<AppSettings>(defaultSettings)

  async function getSettingsPath(): Promise<string> {
    const dataDir = await getLocalDir()
    return await join(dataDir, SETTINGS_FILE_NAME)
  }

  const loadSettings = async () => {
    await runLoggedTask(
      'settings.load',
      async () => {
        const settingsPath = await getSettingsPath()
        // if not exists, create default settings
        if (!(await exists(settingsPath))) {
          settings.value = defaultSettings
          await saveSettings()
          return {
            settingsPath,
            language: settings.value.language ?? 'system',
            created: true
          }
        }

        const settingsContent = await readTextFile(settingsPath)
        const settingsJson = JSON.parse(settingsContent)
        // validation
        if (settingsJson.version !== '1') {
          throw new Error(`Invalid settings file version ${settingsJson.version}`)
        }
        settings.value = normalizeSettings(settingsJson)

        return {
          settingsPath,
          language: settings.value.language ?? 'system',
          created: false
        }
      },
      {
        start: `load file=${SETTINGS_FILE_NAME}`,
        success: ({ settingsPath, language, created }) =>
          `${created ? 'created default' : 'loaded'} file=${settingsPath} locale=${language}`
      }
    )
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
    try {
      await writeTextFile(settingsPath, settingsContent)
      logFrontendDebug(
        'settings.save',
        `saved file=${settingsPath} mode=${byAutoSave ? 'auto' : 'manual'}`
      )
    } catch (error) {
      logFrontendError('settings.save', `save failed file=${settingsPath}`, error)
      throw error
    }
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
