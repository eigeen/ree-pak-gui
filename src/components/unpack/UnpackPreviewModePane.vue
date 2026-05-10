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
    accent?: 'audio' | 'model' | 'default'
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
  <div
    class="flex h-full min-w-0 flex-col [--preview-mode-accent:#5dccd0] [--preview-mode-bg:#15282d] [--preview-mode-chip-bg:transparent] [--preview-mode-fg:#a8b3b6] [--preview-mode-filename-fg:#fff]"
    :data-preview-accent="props.accent"
  >
    <div
      class="flex items-center gap-2.5 bg-[var(--preview-mode-bg)] px-3 py-[5px] text-xs leading-[1.2] text-[var(--preview-mode-fg)]"
    >
      <button
        type="button"
        class="inline-flex shrink-0 cursor-pointer items-center gap-1 rounded px-2 py-1 font-normal text-[#cccccc] transition-colors hover:bg-white/[0.06] hover:text-[var(--preview-mode-accent)]"
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
          <span class="truncate text-xs text-[var(--preview-mode-fg)]">{{ segment.label }}</span>
          <ChevronRight class="size-3 shrink-0 opacity-50" />
        </template>

        <span
          class="inline-flex min-w-0 items-center gap-1 rounded bg-[var(--preview-mode-chip-bg)] px-2 py-1 font-semibold text-[var(--preview-mode-filename-fg)] [&_svg]:text-[var(--preview-mode-accent)]"
        >
          <component v-if="props.fileIcon" :is="props.fileIcon" class="size-3 shrink-0" />
          <span class="truncate">{{ props.fileName }}</span>
        </span>
      </div>

      <span
        v-if="props.kindLabel"
        class="inline-flex shrink-0 items-center gap-1 rounded-[10px] border border-[var(--preview-mode-accent)] bg-[var(--preview-mode-chip-bg)] px-2 py-1 text-[11px] font-semibold text-[var(--preview-mode-filename-fg)] [&_svg]:text-[var(--preview-mode-accent)]"
      >
        <component v-if="props.fileIcon" :is="props.fileIcon" class="size-3 shrink-0" />
        {{ props.kindLabel }}
      </span>
    </div>

    <div class="h-0.5 shrink-0 bg-[var(--preview-mode-accent)]" aria-hidden="true" />

    <div class="relative min-h-0 flex-1">
      <slot />
    </div>
  </div>
</template>

<style scoped>
[data-preview-accent='audio'] {
  --preview-mode-bg: #221912;
  --preview-mode-fg: #888888;
  --preview-mode-accent: #ffad66;
  --preview-mode-chip-bg: #3d2f1c;
  --preview-mode-filename-fg: #ffd3a9;
  --preview-mode-separator: #666666;
}

[data-preview-accent='model'] {
  --preview-mode-bg: #111e22;
  --preview-mode-fg: #9ca8ac;
  --preview-mode-accent: #65d4bf;
  --preview-mode-chip-bg: #172b2d;
  --preview-mode-filename-fg: #c8fff4;
  --preview-mode-separator: #607076;
}
</style>
