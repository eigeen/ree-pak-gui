<script setup lang="ts">
import { ref, watch } from 'vue'
import { AlertTriangle, ChevronDown, ChevronRight, Trash2 } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import type { ConflictFile } from '@/lib/packer'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'

const conflicts = defineModel<ConflictFile[]>('conflicts')
const { t, locale } = useI18n()

const expandedFiles = ref<Set<string>>(new Set())
const localConflicts = ref<ConflictFile[]>([])

watch(
  conflicts,
  (newConflicts) => {
    if (!newConflicts) return

    localConflicts.value = newConflicts.map((conflict) => ({
      ...conflict,
      selectedSourceId:
        conflict.selectedSourceId === undefined
          ? (conflict.sources.at(-1)?.id ?? null)
          : conflict.selectedSourceId
    }))
    expandedFiles.value = new Set(newConflicts.map((conflict) => conflict.targetKey))
  },
  { immediate: true }
)

const toggleExpanded = (targetKey: string) => {
  if (expandedFiles.value.has(targetKey)) {
    expandedFiles.value.delete(targetKey)
  } else {
    expandedFiles.value.add(targetKey)
  }
}

const isExpanded = (targetKey: string) => expandedFiles.value.has(targetKey)

const syncConflicts = () => {
  conflicts.value = localConflicts.value.map((conflict) => ({
    ...conflict
  }))
}

const updateSelectedSource = (conflict: ConflictFile, value: unknown) => {
  const nextValue = String(value)
  conflict.selectedSourceId = nextValue === '__remove__' ? null : nextValue
  syncConflicts()
}

const formatFileSize = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  const index = Math.floor(Math.log(bytes) / Math.log(1024))
  const value = bytes / 1024 ** index
  return `${index === 0 ? Math.round(value) : value.toFixed(2)} ${units[index]}`
}

const formatDate = (date: Date) =>
  date.toLocaleString(locale.value === 'zh_CN' ? 'zh-CN' : 'en-US', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
</script>

<template>
  <div class="space-y-4">
    <p class="section-copy">{{ t('fileConflict.description') }}</p>

    <div class="editor-scrollbar max-h-120 overflow-y-auto">
      <div
        v-for="conflict in localConflicts"
        :key="conflict.targetKey"
        class="border-b border-border/60 px-4 py-3 last:border-b-0"
      >
        <div class="flex items-start gap-3">
          <Button
            size="icon-sm"
            variant="ghost"
            class="mt-0.5 h-7 w-7 rounded-full"
            @click="toggleExpanded(conflict.targetKey)"
          >
            <ChevronDown v-if="isExpanded(conflict.targetKey)" class="size-4" />
            <ChevronRight v-else class="size-4" />
          </Button>

          <div class="flex min-w-0 flex-1 items-start gap-3">
            <div class="flex h-8 w-8 shrink-0 items-center justify-center text-destructive">
              <AlertTriangle class="size-4" />
            </div>

            <div class="min-w-0 flex-1">
              <div class="flex flex-wrap items-center gap-2">
                <p class="break-all text-sm font-semibold text-foreground">
                  {{ conflict.targetPath }}
                </p>
                <Badge variant="outline">
                  {{ t('fileConflict.sourcesCount', { count: conflict.sources.length }) }}
                </Badge>
              </div>
              <div class="mt-1 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
                <span v-if="conflict.size">
                  {{ t('fileConflict.sizeLabel', { size: formatFileSize(conflict.size) }) }}
                </span>
                <span v-if="conflict.modifiedDate">
                  {{ t('fileConflict.modifiedAt', { date: formatDate(conflict.modifiedDate) }) }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <div v-if="isExpanded(conflict.targetKey)" class="ml-9 mt-3 pl-4">
          <RadioGroup
            :model-value="conflict.selectedSourceId ?? '__remove__'"
            class="space-y-1.5"
            @update:model-value="(value) => updateSelectedSource(conflict, value)"
          >
            <label
              class="flex cursor-pointer items-start gap-3 rounded-md px-2 py-2 transition-colors hover:bg-[color-mix(in_oklch,var(--surface-toolbar)_68%,transparent)]"
              :for="`conflict-${conflict.targetKey}-remove`"
            >
              <RadioGroupItem
                :id="`conflict-${conflict.targetKey}-remove`"
                value="__remove__"
                class="mt-0.5"
              />
              <div class="flex min-w-0 items-center gap-2 text-sm font-medium text-destructive">
                <Trash2 class="size-4 shrink-0" />
                <span>{{ t('fileConflict.removeFile') }}</span>
              </div>
            </label>

            <label
              v-for="(source, sourceIndex) in conflict.sources"
              :key="source.id"
              class="flex cursor-pointer items-start gap-3 rounded-md px-2 py-2 transition-colors hover:bg-[color-mix(in_oklch,var(--surface-toolbar)_68%,transparent)]"
              :for="`conflict-${conflict.targetKey}-${sourceIndex}`"
            >
              <RadioGroupItem
                :id="`conflict-${conflict.targetKey}-${sourceIndex}`"
                :value="source.id"
                class="mt-0.5"
              />
              <div class="min-w-0 flex-1">
                <p class="break-all text-sm text-foreground">{{ source.sourcePath }}</p>
              </div>
            </label>
          </RadioGroup>
        </div>
      </div>
    </div>
  </div>
</template>
