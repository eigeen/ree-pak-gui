<script setup lang="ts">
import { ref, watch } from 'vue'
import { AlertTriangle, ChevronDown, ChevronRight, Trash2 } from 'lucide-vue-next'
import type { ConflictFile } from '@/lib/packer'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'

const conflicts = defineModel<ConflictFile[]>('conflicts')

const expandedFiles = ref<Set<string>>(new Set())
const localConflicts = ref<ConflictFile[]>([])

watch(
  conflicts,
  (newConflicts) => {
    if (!newConflicts) return

    localConflicts.value = newConflicts.map((conflict) => ({
      ...conflict,
      selectedSource: conflict.selectedSource ?? conflict.sources.length - 1
    }))
  },
  { immediate: true }
)

const toggleExpanded = (relativePath: string) => {
  if (expandedFiles.value.has(relativePath)) {
    expandedFiles.value.delete(relativePath)
  } else {
    expandedFiles.value.add(relativePath)
  }
}

const isExpanded = (relativePath: string) => expandedFiles.value.has(relativePath)

const formatFileSize = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  const index = Math.floor(Math.log(bytes) / Math.log(1024))
  const value = bytes / 1024 ** index
  return `${index === 0 ? Math.round(value) : value.toFixed(2)} ${units[index]}`
}

const formatDate = (date: Date) =>
  date.toLocaleString('zh-CN', {
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
    <p class="section-copy">检测到以下文件存在冲突，请选择要保留的文件版本。</p>

    <div class="editor-scrollbar max-h-[30rem] space-y-3 overflow-y-auto pr-2">
      <div
        v-for="conflict in localConflicts"
        :key="conflict.relativePath"
        class="rounded-[1.25rem] border border-border/70 bg-secondary/25 p-4"
      >
        <div class="flex items-start gap-3">
          <Button
            size="icon-sm"
            variant="ghost"
            class="mt-0.5 rounded-full"
            @click="toggleExpanded(conflict.relativePath)"
          >
            <ChevronDown v-if="isExpanded(conflict.relativePath)" class="size-4" />
            <ChevronRight v-else class="size-4" />
          </Button>

          <div
            class="flex size-9 shrink-0 items-center justify-center rounded-2xl border border-destructive/25 bg-destructive/10 text-destructive"
          >
            <AlertTriangle class="size-4" />
          </div>

          <div class="min-w-0 flex-1">
            <div class="flex flex-wrap items-center gap-2">
              <p class="break-all text-sm font-semibold text-foreground">
                {{ conflict.relativePath }}
              </p>
              <Badge variant="outline">{{ conflict.sources.length }} 个来源</Badge>
            </div>
            <div class="mt-1 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
              <span v-if="conflict.size">大小: {{ formatFileSize(conflict.size) }}</span>
              <span v-if="conflict.modifiedDate">
                修改时间: {{ formatDate(conflict.modifiedDate) }}
              </span>
            </div>
          </div>
        </div>

        <div
          v-if="isExpanded(conflict.relativePath)"
          class="mt-4 space-y-2 border-l border-border pl-5"
        >
          <RadioGroup
            :model-value="String(conflict.selectedSource)"
            class="space-y-2"
            @update:model-value="(value) => (conflict.selectedSource = Number(value))"
          >
            <label
              class="flex cursor-pointer items-start gap-3 rounded-2xl border border-border/70 bg-background/90 px-3 py-3"
              :for="`conflict-${conflict.relativePath}-remove`"
            >
              <RadioGroupItem
                :id="`conflict-${conflict.relativePath}-remove`"
                value="-1"
                class="mt-1"
              />
              <div class="flex items-center gap-2 text-sm font-medium text-destructive">
                <Trash2 class="size-4" />
                <span>移除该文件</span>
              </div>
            </label>

            <label
              v-for="(source, index) in conflict.sources"
              :key="index"
              class="flex cursor-pointer items-start gap-3 rounded-2xl border border-border/70 bg-background/90 px-3 py-3"
              :for="`conflict-${conflict.relativePath}-${index}`"
            >
              <RadioGroupItem
                :id="`conflict-${conflict.relativePath}-${index}`"
                :value="String(index)"
                class="mt-1"
              />
              <div class="min-w-0 flex-1">
                <p class="break-all text-sm font-medium text-foreground">{{ source.sourcePath }}</p>
              </div>
            </label>
          </RadioGroup>
        </div>
      </div>
    </div>
  </div>
</template>
