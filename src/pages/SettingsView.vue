<template>
  <section class="desktop-page">
    <div class="flex min-h-0 flex-1 flex-col">
      <header class="border-b border-border/80 bg-secondary/45 px-5 py-3 backdrop-blur-sm">
        <div class="mb-3 flex items-center gap-2 text-sm font-medium text-foreground">
          <Settings2 class="size-4 text-muted-foreground" />
          <span>{{ t('settings.title') }}</span>
        </div>

        <div class="relative max-w-none">
          <Search
            class="pointer-events-none absolute left-2.5 top-2 size-4 text-muted-foreground"
          />
          <DenseInput
            v-model="searchText"
            type="text"
            class="pl-8"
            :placeholder="t('settings.searchPlaceholder')"
          />
        </div>
      </header>

      <div class="grid min-h-0 flex-1 grid-cols-[15.5rem_minmax(0,1fr)]">
        <aside
          class="editor-scrollbar overflow-auto border-r border-border/80 bg-secondary/30 px-3 py-3"
        >
          <div v-for="section in filteredSections" :key="section.id" class="mb-1">
            <button
              type="button"
              class="flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left text-sm transition-colors"
              :class="
                activeSection === section.id
                  ? 'bg-accent text-accent-foreground shadow-sm'
                  : 'text-muted-foreground hover:bg-secondary/70 hover:text-foreground'
              "
              @click="scrollToSection(section.id)"
            >
              <ChevronRight class="size-4 shrink-0" />
              <span class="truncate">{{ section.label }}</span>
            </button>
          </div>
        </aside>

        <main class="editor-scrollbar overflow-auto bg-background/10 px-9 py-6">
          <div class="mx-auto max-w-4xl">
            <div class="mb-8 flex items-start justify-between gap-4 border-b border-border/70 pb-4">
              <div>
                <h2 class="text-3xl font-semibold leading-none text-foreground">
                  {{ t('settings.globalTitle') }}
                </h2>
                <p class="mt-4 text-sm text-muted-foreground">
                  {{ t('settings.globalDescription') }}
                </p>
              </div>
            </div>

            <div v-if="filteredSections.length > 0" class="space-y-10">
              <section
                v-for="section in filteredSections"
                :id="`settings-section-${section.id}`"
                :key="section.id"
                class="scroll-mt-6 border-b border-border/70 pb-8 last:border-b-0"
              >
                <div class="mb-4">
                  <h3 class="text-xl font-semibold text-foreground">{{ section.label }}</h3>
                </div>

                <template v-if="section.id === 'common'">
                  <div class="max-w-3xl space-y-8">
                    <div v-if="hasVisibleGroup(section, 'language')">
                      <div class="mb-3">
                        <h4 class="text-base font-semibold text-foreground">
                          {{ getSectionGroup(section, 'language')?.title }}
                        </h4>
                      </div>

                      <SettingsInlineItem
                        v-if="hasVisibleItem(section, 'language', 'language')"
                        :title="t('settings.languageTitle')"
                        :description="t('settings.languageDescription')"
                      >
                        <LanguageSelect />
                      </SettingsInlineItem>
                    </div>

                    <div v-if="hasVisibleGroup(section, 'theme')">
                      <div class="mb-3">
                        <h4 class="text-base font-semibold text-foreground">
                          {{ getSectionGroup(section, 'theme')?.title }}
                        </h4>
                      </div>

                      <SettingsInlineItem
                        v-if="hasVisibleItem(section, 'theme', 'theme')"
                        :title="t('settings.themeTitle')"
                        :description="t('settings.themeDescription')"
                      >
                        <Select v-model="themeMode">
                          <SelectTrigger class="w-full max-w-52">
                            <SelectValue :placeholder="t('settings.themeTitle')" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem
                              v-for="mode in themeModes"
                              :key="mode.value"
                              :value="mode.value"
                            >
                              {{ mode.label }}
                            </SelectItem>
                          </SelectContent>
                        </Select>
                        <p class="mt-3 text-sm text-muted-foreground">
                          {{
                            t('settings.themeCurrentDescription', {
                              current: isDark
                                ? t('settings.themeModeDark')
                                : t('settings.themeModeLight')
                            })
                          }}
                        </p>
                      </SettingsInlineItem>
                    </div>
                  </div>
                </template>

                <template v-else-if="section.id === 'file-explorer'">
                  <div class="max-w-3xl space-y-8">
                    <div v-if="hasVisibleGroup(section, 'preview')">
                      <div class="mb-3">
                        <h4 class="text-base font-semibold text-foreground">
                          {{ getSectionGroup(section, 'preview')?.title }}
                        </h4>
                      </div>

                      <SettingsInlineItem
                        v-if="hasVisibleItem(section, 'preview', 'texture-preview')"
                        :description="t('settings.texturePreviewDescription')"
                      >
                        <template #title>
                          <p class="text-sm font-semibold text-foreground">
                            {{ t('settings.texturePreviewTitle') }}
                          </p>
                          <TooltipProvider>
                            <Tooltip>
                              <TooltipTrigger as-child>
                                <button
                                  type="button"
                                  class="inline-flex size-4 items-center justify-center text-muted-foreground transition-colors hover:text-foreground"
                                >
                                  <CircleAlert class="size-4" />
                                </button>
                              </TooltipTrigger>
                              <TooltipContent>
                                {{ t('settings.texturePreviewHint') }}
                              </TooltipContent>
                            </Tooltip>
                          </TooltipProvider>
                        </template>
                        <label class="inline-flex items-center gap-3">
                          <Switch v-model="showTexturePreview" />
                          <span class="text-sm text-foreground">
                            {{
                              showTexturePreview ? t('settings.enabled') : t('settings.disabled')
                            }}
                          </span>
                        </label>
                      </SettingsInlineItem>
                    </div>
                  </div>
                </template>

                <template v-else>
                  <div class="text-sm text-muted-foreground">
                    {{ t('settings.placeholderSection') }}
                  </div>
                </template>
              </section>
            </div>

            <div
              v-else
              class="rounded-md border border-dashed border-border/70 px-4 py-10 text-sm text-muted-foreground"
            >
              {{ t('settings.searchEmpty') }}
            </div>
          </div>
        </main>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, unref, watch, type Ref } from 'vue'
import { ChevronRight, CircleAlert, Search, Settings2 } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import SettingsInlineItem from '@/components/Settings/SettingsInlineItem.vue'
import LanguageSelect from '@/components/LanguageSelect.vue'
import { DenseInput } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { APP_LOCALE_OPTIONS } from '@/lib/language'
import { useSettingsStore, type AppSettings, type ThemeMode } from '@/store/settings'
import { useAppTheme } from '@/composables/theme'

type SettingsItem = {
  id: string
  title: string
  description?: string
  keywords?: string[]
}

type SettingsGroup = {
  id: string
  title: string
  items: SettingsItem[]
}

type SettingsSection = {
  id: string
  label: string
  groups: SettingsGroup[]
}

type ThemeOption = {
  value: ThemeMode
  label: string
}

const { t } = useI18n()
const settingsStore = useSettingsStore()
const settings = computed(() => unref(settingsStore.settings as unknown as Ref<AppSettings>))
const { isDark, themeMode } = useAppTheme()

const searchText = ref('')
const activeSection = ref('common')

const themeModes = computed<ThemeOption[]>(() => [
  { value: 'system', label: t('settings.themeModeSystem') },
  { value: 'light', label: t('settings.themeModeLight') },
  { value: 'dark', label: t('settings.themeModeDark') }
])

const sections = computed<SettingsSection[]>(() => [
  {
    id: 'common',
    label: t('settings.sectionCommon'),
    groups: [
      {
        id: 'language',
        title: t('settings.languageSection'),
        items: [
          {
            id: 'language',
            title: t('settings.languageTitle'),
            description: t('settings.languageDescription'),
            keywords: APP_LOCALE_OPTIONS.map((option) => option.label)
          }
        ]
      },
      {
        id: 'theme',
        title: t('settings.themeSection'),
        items: [
          {
            id: 'theme',
            title: t('settings.themeTitle'),
            description: t('settings.themeDescription'),
            keywords: themeModes.value.map((mode) => mode.label)
          }
        ]
      }
    ]
  },
  {
    id: 'file-explorer',
    label: t('settings.sectionFileExplorer'),
    groups: [
      {
        id: 'preview',
        title: t('settings.previewTitle'),
        items: [
          {
            id: 'texture-preview',
            title: t('settings.texturePreviewTitle'),
            description: t('settings.texturePreviewDescription'),
            keywords: [
              t('settings.texturePreviewHint'),
              t('settings.enabled'),
              t('settings.disabled')
            ]
          }
        ]
      }
    ]
  }
])

const filteredSections = computed(() => {
  const keyword = searchText.value.trim().toLowerCase()
  if (!keyword) {
    return sections.value
  }

  return sections.value
    .map((section) => {
      const isSectionMatched = includesKeyword([section.label], keyword)
      const groups = section.groups
        .map((group) => {
          const isGroupMatched = includesKeyword([group.title], keyword)
          const items =
            isSectionMatched || isGroupMatched
              ? group.items
              : group.items.filter((item) =>
                  includesKeyword([item.title, item.description, ...(item.keywords ?? [])], keyword)
                )

          if (items.length === 0) {
            return null
          }

          return {
            ...group,
            items
          }
        })
        .filter((group): group is SettingsGroup => group !== null)

      if (groups.length === 0) {
        return null
      }

      return {
        ...section,
        groups
      }
    })
    .filter((section): section is SettingsSection => section !== null)
})

const showTexturePreview = computed({
  get: () => settings.value?.preview?.showTexturePreview ?? true,
  set: (value: boolean) => {
    if (!settings.value?.preview) {
      return
    }

    settings.value.preview.showTexturePreview = value
  }
})

watch(
  filteredSections,
  (nextSections) => {
    if (nextSections.length === 0) {
      return
    }

    if (!nextSections.some((section) => section.id === activeSection.value)) {
      activeSection.value = nextSections[0]?.id ?? 'common'
    }
  },
  { immediate: true }
)

function scrollToSection(sectionId: string) {
  activeSection.value = sectionId
  document
    .getElementById(`settings-section-${sectionId}`)
    ?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function includesKeyword(values: Array<string | undefined>, keyword: string) {
  return values.some((value) => value?.toLowerCase().includes(keyword))
}

function getSectionGroup(section: SettingsSection, groupId: string) {
  return section.groups.find((group) => group.id === groupId)
}

function hasVisibleGroup(section: SettingsSection, groupId: string) {
  return !!getSectionGroup(section, groupId)
}

function hasVisibleItem(section: SettingsSection, groupId: string, itemId: string) {
  return !!getSectionGroup(section, groupId)?.items.some((item) => item.id === itemId)
}
</script>
