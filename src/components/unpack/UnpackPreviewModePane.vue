<script setup lang="ts">
import { ArrowLeft, ChevronRight, Folder } from 'lucide-vue-next'
import { onMounted, onUnmounted, type Component } from 'vue'

type Segment = {
  id?: string
  label: string
}

const props = withDefaults(
  defineProps<{
    exitLabel: string
    kindLabel: string
    fileName: string
    fileIcon?: Component | null
    parentSegments?: Segment[]
    accent?: 'audio' | 'default'
  }>(),
  {
    fileIcon: null,
    parentSegments: () => [],
    accent: 'default'
  }
)

const emit = defineEmits<{
  (e: 'exit'): void
}>()

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault()
    emit('exit')
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="flex h-full min-w-0 flex-col" :data-preview-accent="props.accent">
    <div class="preview-mode-breadcrumb">
      <button
        type="button"
        class="preview-mode-exit"
        :title="props.exitLabel"
        :aria-label="props.exitLabel"
        @click="emit('exit')"
      >
        <ArrowLeft class="size-3.5" />
        <span>{{ props.exitLabel }}</span>
      </button>

      <span class="preview-mode-divider" aria-hidden="true" />

      <div class="flex min-w-0 flex-1 items-center gap-1.5 overflow-hidden">
        <template
          v-for="(segment, index) in props.parentSegments"
          :key="`${segment.label}-${index}`"
        >
          <Folder v-if="index === 0" class="size-3 shrink-0 opacity-80" />
          <span class="truncate text-[11px] opacity-90">{{ segment.label }}</span>
          <ChevronRight class="size-3 shrink-0 opacity-70" />
        </template>

        <component v-if="props.fileIcon" :is="props.fileIcon" class="size-3 shrink-0" />
        <span class="preview-mode-filename truncate">{{ props.fileName }}</span>
      </div>

      <span v-if="props.kindLabel" class="preview-mode-kind">{{ props.kindLabel }}</span>
    </div>

    <div class="relative min-h-0 flex-1">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.preview-mode-breadcrumb {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  font-size: 11px;
  line-height: 1.2;
  background: var(--preview-mode-bg, #0d3a42);
  color: var(--preview-mode-fg, #a8eef0);
  border-bottom: 1px solid var(--preview-mode-border, #1a5560);
}

[data-preview-accent='audio'] .preview-mode-breadcrumb {
  --preview-mode-bg: #0d3a42;
  --preview-mode-fg: #a8eef0;
  --preview-mode-border: #1a5560;
  --preview-mode-accent: #5dccd0;
  --preview-mode-filename-fg: #ffffff;
}

.preview-mode-exit {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-weight: 600;
  color: var(--preview-mode-fg);
  transition: color 0.15s ease;
  cursor: pointer;
}

.preview-mode-exit:hover {
  color: var(--preview-mode-accent, var(--preview-mode-fg));
}

.preview-mode-divider {
  width: 1px;
  height: 16px;
  background: var(--preview-mode-border);
}

.preview-mode-filename {
  font-weight: 600;
  color: var(--preview-mode-filename-fg, var(--preview-mode-fg));
}

.preview-mode-kind {
  flex-shrink: 0;
  font-weight: 500;
  opacity: 0.85;
}
</style>
