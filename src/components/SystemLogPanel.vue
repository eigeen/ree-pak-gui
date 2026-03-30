<template>
  <div class="surface-console flex h-full min-w-0 flex-col">
    <div
      ref="consoleContainer"
      class="surface-console-panel text-2xs editor-scrollbar min-h-0 min-w-0 flex-1 overflow-auto border border-border/60 px-3 py-2 font-mono"
    >
      <div
        v-for="line in consoleLines"
        :key="line.id"
        class="min-w-0 whitespace-pre-wrap break-all"
      >
        <span class="log-timestamp">[{{ formatLogTime(line.createdAt) }}]</span>
        <span class="mx-1 font-semibold" :class="getLogLevelClass(line.level)">
          {{ getLogLevelLabel(line.level) }}
        </span>
        <span :class="getLogMessageClass(line.level)">{{ line.message }}</span>
      </div>

      <div v-if="consoleLines.length === 0" class="flex h-full items-center text-muted-foreground">
        {{ emptyText || 'No system logs yet' }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, shallowRef, toRef, watch } from 'vue'
import { useSystemLogStore, type SystemLogEntry, type SystemLogLevel } from '@/store/system'

const props = withDefaults(
  defineProps<{
    eyebrow?: string
    emptyText?: string
    maxEntries?: number
  }>(),
  {
    eyebrow: '',
    emptyText: '',
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
      return 'log-level-error'
    case 'warn':
      return 'log-level-warn'
    case 'info':
      return 'log-level-info'
    case 'debug':
      return 'log-level-debug'
  }
}

function getLogMessageClass(level: SystemLogLevel) {
  switch (level) {
    case 'error':
      return 'log-message-error'
    case 'warn':
      return 'log-message-warn'
    case 'info':
      return 'log-message-info'
    case 'debug':
      return 'log-message-debug'
  }
}
</script>

<style scoped>
.log-timestamp {
  color: var(--log-timestamp);
}

.log-level-error {
  color: var(--log-level-error);
}

.log-level-warn {
  color: var(--log-level-warn);
}

.log-level-info {
  color: var(--log-level-info);
}

.log-level-debug {
  color: var(--log-level-debug);
}

.log-message-error {
  color: var(--log-message-error);
}

.log-message-warn {
  color: var(--log-message-warn);
}

.log-message-info {
  color: var(--log-message-info);
}

.log-message-debug {
  color: var(--log-message-debug);
}
</style>
