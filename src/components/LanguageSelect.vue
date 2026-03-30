<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button
        :variant="compact ? 'ghost' : 'outline'"
        :size="compact ? 'icon-sm' : 'default'"
        :class="compact ? 'desktop-icon-button' : 'min-w-40 justify-between gap-2'"
        :title="triggerTitle"
      >
        <Languages v-if="compact" class="size-4" />
        <template v-else>
          <span>{{ currentLanguageOption.label }}</span>
          <ChevronDown class="size-4 text-muted-foreground" />
        </template>
      </Button>
    </DropdownMenuTrigger>

    <DropdownMenuContent align="end" class="w-40">
      <DropdownMenuRadioGroup v-model="selectedLanguage">
        <DropdownMenuRadioItem
          v-for="option in APP_LOCALE_OPTIONS"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </DropdownMenuRadioItem>
      </DropdownMenuRadioGroup>
    </DropdownMenuContent>
  </DropdownMenu>
</template>

<script setup lang="ts">
import { computed, unref, type Ref } from 'vue'
import { ChevronDown, Languages } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { APP_LOCALE_OPTIONS, resolveLocale } from '@/lib/language'
import { useSettingsStore, type AppSettings } from '@/store/settings'

interface Props {
  compact?: boolean
  title?: string
}

const props = withDefaults(defineProps<Props>(), {
  compact: false,
  title: ''
})

const { locale } = useI18n()
const settingsStore = useSettingsStore()
const settings = computed(() => unref(settingsStore.settings as unknown as Ref<AppSettings>))

const selectedLanguage = computed({
  get: () => resolveLocale(settings.value?.language),
  set: (value) => {
    settings.value.language = value
    locale.value = value
  }
})

const currentLanguageOption = computed(() => {
  return (
    APP_LOCALE_OPTIONS.find((option) => option.value === selectedLanguage.value) ??
    APP_LOCALE_OPTIONS[0]
  )
})

const triggerTitle = computed(() => props.title || currentLanguageOption.value.label)
</script>
