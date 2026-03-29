<template>
  <div class="surface-console flex h-full min-w-0 flex-col">
    <div class="desktop-toolbar h-10 justify-between px-3">
      <div>
        <p v-if="eyebrow" class="section-eyebrow">{{ eyebrow }}</p>
        <h3 class="section-title">{{ title }}</h3>
      </div>
    </div>

    <div
      ref="consoleContainer"
      class="surface-console-panel text-2xs editor-scrollbar min-h-0 min-w-0 flex-1 overflow-auto border border-border/60 px-3 py-2 font-mono"
    >
      <div
        v-for="line in consoleLines"
        :key="line.id"
        class="min-w-0 whitespace-pre-wrap break-all"
      >
        <span class="text-muted-foreground/70">[{{ formatLogTime(line.createdAt) }}]</span>
        <span class="mx-1 font-semibold" :class="getLogLevelClass(line.level)">
          {{ getLogLevelLabel(line.level) }}
        </span>
        <span :class="getLogMessageClass(line.level)">{{ line.message }}</span>
      </div>

      <div v-if="consoleLines.length === 0" class="flex h-full items-center text-muted-foreground">
        {{ emptyText }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, shallowRef, toRef, watch } from 'vue'
import { useSystemLogStore, type SystemLogEntry, type SystemLogLevel } from '@/store/system'

const props = withDefaults(
  defineProps<{
    title?: string
    eyebrow?: string
    emptyText?: string
    maxEntries?: number
  }>(),
  {
    title: 'System Log',
    eyebrow: '',
    emptyText: '暂无 system 日志',
    maxEntries: 160
  }
)

const systemLogStore = useSystemLogStore()
const systemLogEntries = toRef(systemLogStore, 'entries')
const isProductionBuild = import.meta.env.PROD
const consoleLines = shallowRef<SystemLogEntry[]>([])
const consoleContainer = ref<HTMLElement | null>(null)
const entryLimit = computed(() => Math.max(1, props.maxEntries))

watch(
  systemLogEntries,
  async (entries) => {
    const shouldStickToBottom = isConsoleNearBottom()
    const visibleEntries = isProductionBuild
      ? entries.filter((entry) => entry.level !== 'debug')
      : entries

    consoleLines.value = visibleEntries.slice(-entryLimit.value)

    if (!shouldStickToBottom) return

    await nextTick()
    scrollConsoleToBottom()
  },
  { immediate: true }
)

function formatLogTime(value: string) {
  return new Date(value).toLocaleTimeString()
}

function isConsoleNearBottom() {
  const element = consoleContainer.value
  if (!element) return true

  const distanceToBottom = element.scrollHeight - element.scrollTop - element.clientHeight
  return distanceToBottom <= 24
}

function scrollConsoleToBottom() {
  const element = consoleContainer.value
  if (!element) return

  element.scrollTop = element.scrollHeight
}

function getLogLevelLabel(level: SystemLogLevel) {
  switch (level) {
    case 'error':
      return '[ERROR]'
    case 'warn':
      return '[WARN]'
    case 'info':
      return '[INFO]'
    case 'debug':
      return '[DEBUG]'
  }
}

function getLogLevelClass(level: SystemLogLevel) {
  switch (level) {
    case 'error':
      return 'text-destructive'
    case 'warn':
      return 'text-amber-400'
    case 'info':
      return 'text-sky-400'
    case 'debug':
      return 'text-emerald-400'
  }
}

function getLogMessageClass(level: SystemLogLevel) {
  switch (level) {
    case 'error':
      return 'text-destructive'
    case 'warn':
      return 'text-amber-200'
    case 'info':
      return 'text-foreground'
    case 'debug':
      return 'text-muted-foreground'
  }
}
</script>
