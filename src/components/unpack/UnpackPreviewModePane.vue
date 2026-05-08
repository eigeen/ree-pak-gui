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

      <div class="flex min-w-0 flex-1 items-center gap-1.5 overflow-hidden">
        <template
          v-for="(segment, index) in props.parentSegments"
          :key="`${segment.label}-${index}`"
        >
          <Folder v-if="index === 0" class="size-3 shrink-0 opacity-70" />
          <span class="preview-mode-segment truncate">{{ segment.label }}</span>
          <ChevronRight class="size-3 shrink-0 opacity-50" />
        </template>

        <span class="preview-mode-filename-chip">
          <component
            v-if="props.fileIcon"
            :is="props.fileIcon"
            class="size-3 shrink-0"
          />
          <span class="truncate">{{ props.fileName }}</span>
        </span>
      </div>

      <span v-if="props.kindLabel" class="preview-mode-kind-badge">
        <component
          v-if="props.fileIcon"
          :is="props.fileIcon"
          class="size-3 shrink-0"
        />
        {{ props.kindLabel }}
      </span>
    </div>

    <div class="preview-mode-accent-rule" aria-hidden="true" />

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
  padding: 8px 12px;
  font-size: 12px;
  line-height: 1.2;
  background: var(--preview-mode-bg, #15282d);
  color: var(--preview-mode-fg, #a8b3b6);
}

.preview-mode-accent-rule {
  height: 2px;
  background: var(--preview-mode-accent, #5dccd0);
  flex-shrink: 0;
}

[data-preview-accent='audio'] {
  --preview-mode-bg: #221912;
  --preview-mode-fg: #888888;
  --preview-mode-accent: #ffad66;
  --preview-mode-chip-bg: #3d2f1c;
  --preview-mode-filename-fg: #ffd3a9;
  --preview-mode-separator: #666666;
}

.preview-mode-exit {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  font-weight: normal;
  color: #cccccc;
  transition:
    color 0.15s ease,
    background 0.15s ease;
  cursor: pointer;
  flex-shrink: 0;
}

.preview-mode-exit:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--preview-mode-accent, #5dccd0);
}

.preview-mode-segment {
  font-size: 12px;
  color: var(--preview-mode-fg);
}

.preview-mode-filename-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 4px;
  background: var(--preview-mode-chip-bg, transparent);
  color: var(--preview-mode-filename-fg, #ffffff);
  font-weight: 600;
  min-width: 0;
}

.preview-mode-filename-chip :deep(svg) {
  color: var(--preview-mode-accent);
}

.preview-mode-kind-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 10px;
  background: var(--preview-mode-chip-bg, transparent);
  border: 1px solid var(--preview-mode-accent, #5dccd0);
  color: var(--preview-mode-filename-fg, #ffffff);
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

.preview-mode-kind-badge :deep(svg) {
  color: var(--preview-mode-accent);
}
</style>
