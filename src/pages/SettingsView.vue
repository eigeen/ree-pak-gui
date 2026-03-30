<template>
  <section class="desktop-page">
    <div class="flex min-h-0 flex-1 flex-col">
      <header class="border-b border-border/80 bg-secondary/45 px-5 py-3 backdrop-blur-sm">
        <div class="mb-3 flex items-center gap-2 text-sm font-medium text-foreground">
          <Settings2 class="size-4 text-muted-foreground" />
          <span>{{ t('settings.title') }}</span>
        </div>

        <div class="relative max-w-none">
          <Search class="pointer-events-none absolute left-2.5 top-2 size-4 text-muted-foreground" />
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

            <div class="space-y-10">
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
                    <div>
                      <div class="mb-3">
                        <h4 class="text-base font-semibold text-foreground">
                          {{ t('settings.languageSection') }}
                        </h4>
                      </div>

                      <SettingsInlineItem
                        :title="t('settings.languageTitle')"
                        :description="t('settings.languageDescription')"
                      >
                        <LanguageSelect />
                      </SettingsInlineItem>
                    </div>
                  </div>
                </template>

                <template v-else-if="section.id === 'file-explorer'">
                  <div class="max-w-3xl space-y-8">
                    <div>
                      <div class="mb-3">
                        <h4 class="text-base font-semibold text-foreground">
                          {{ t('settings.previewTitle') }}
                        </h4>
                      </div>

                      <SettingsInlineItem :description="t('settings.texturePreviewDescription')">
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
import { Switch } from '@/components/ui/switch'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { useSettingsStore, type AppSettings } from '@/store/settings'

type SettingsSection = {
  id: string
  label: string
}

const { t } = useI18n()
const settingsStore = useSettingsStore()
const settings = computed(() => unref(settingsStore.settings as unknown as Ref<AppSettings>))

const searchText = ref('')
const activeSection = ref('common')

const sections = computed<SettingsSection[]>(() => [
  { id: 'common', label: t('settings.sectionCommon') },
  { id: 'file-explorer', label: t('settings.sectionFileExplorer') }
])

const filteredSections = computed(() => {
  const keyword = searchText.value.trim().toLowerCase()
  if (!keyword) {
    return sections.value
  }

  return sections.value.filter((section) => section.label.toLowerCase().includes(keyword))
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
</script>
