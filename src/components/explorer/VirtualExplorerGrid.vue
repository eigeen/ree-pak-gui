<script setup lang="ts" generic="TItem extends { id: string }">
import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

const props = withDefaults(defineProps<{
  items: TItem[]
  selectedKey?: string
  resetKey?: string | number
  minColumnWidth?: number
  gap?: number
  overscan?: number
  cardHeight?: number
}>(), {
  selectedKey: '',
  resetKey: '',
  minColumnWidth: 144,
  gap: 12,
  overscan: 3,
  cardHeight: 236
})

const emit = defineEmits<{
  (e: 'item-click', item: TItem): void
  (e: 'item-open', item: TItem): void
  (e: 'item-contextmenu', item: TItem, event: MouseEvent): void
  (e: 'background-contextmenu', event: MouseEvent): void
  (e: 'visible-items-change', items: TItem[]): void
}>()

const scrollElementRef = ref<HTMLElement | null>(null)
const containerWidth = ref(0)

let resizeObserver: ResizeObserver | null = null

const columnCount = computed(() => {
  const width = Math.max(containerWidth.value, props.minColumnWidth)
  return Math.max(1, Math.floor((width + props.gap) / (props.minColumnWidth + props.gap)))
})

const rows = computed(() => {
  const nextRows: TItem[][] = []
  const step = columnCount.value

  for (let index = 0; index < props.items.length; index += step) {
    nextRows.push(props.items.slice(index, index + step))
  }

  return nextRows
})

const rowHeight = computed(() => props.cardHeight + props.gap)
const rowGridStyle = computed(() => ({
  gridTemplateColumns: `repeat(${columnCount.value}, minmax(0, 1fr))`
}))

const virtualizer = useVirtualizer<HTMLElement, HTMLDivElement>(
  computed(() => ({
    count: rows.value.length,
    getScrollElement: () => scrollElementRef.value,
    estimateSize: () => rowHeight.value,
    overscan: props.overscan,
    getItemKey: (index: number) => `explorer-row-${index}`
  }))
)

const virtualRows = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
const visibleItems = computed(() =>
  virtualRows.value.flatMap((virtualRow) => rows.value[virtualRow.index] ?? [])
)

watch(visibleItems, (items) => {
  emit('visible-items-change', items)
}, { immediate: true })

watch(columnCount, () => {
  virtualizer.value.measure()
})

watch(() => props.resetKey, () => {
  scrollElementRef.value?.scrollTo({ top: 0, behavior: 'auto' })
  virtualizer.value.scrollToOffset(0)
})

onMounted(() => {
  if (!scrollElementRef.value) return

  const updateWidth = (element: HTMLElement) => {
    containerWidth.value = element.clientWidth
  }

  updateWidth(scrollElementRef.value)

  resizeObserver = new ResizeObserver((entries) => {
    const entry = entries[0]
    const target = entry?.target
    if (!(target instanceof HTMLElement)) return

    updateWidth(target)
  })

  resizeObserver.observe(scrollElementRef.value)
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})

function getRowItems(index: number) {
  return rows.value[index] ?? []
}

function getRowStyle(start: number) {
  return {
    height: `${rowHeight.value}px`,
    transform: `translateY(${start}px)`
  }
}

function getCardClass(item: TItem) {
  const baseClass =
    'explorer-grid-card relative isolate group flex min-h-0 flex-col overflow-hidden rounded-[0.4rem] border-2 text-left transition-[background-color,border-color,box-shadow] duration-150'
  if (props.selectedKey !== item.id) {
    return baseClass
  }

  return `${baseClass} explorer-grid-card-active`
}

function handleBackgroundContextMenu(event: MouseEvent) {
  const target = event.target
  if (target instanceof Element && target.closest('[data-explorer-item-root]')) {
    return
  }

  emit('background-contextmenu', event)
}
</script>

<template>
  <div
    ref="scrollElementRef"
    class="editor-scrollbar h-full overflow-auto px-1.5 py-1.5"
    @contextmenu="handleBackgroundContextMenu"
  >
    <div class="relative min-h-full" :style="{ height: `${totalSize}px` }">
      <div
        v-for="virtualRow in virtualRows"
        :key="String(virtualRow.key)"
        class="absolute left-0 top-0 w-full"
        :style="getRowStyle(virtualRow.start)"
      >
        <div class="grid gap-3" :style="rowGridStyle">
          <button
            v-for="item in getRowItems(virtualRow.index)"
            :key="item.id"
            type="button"
            data-explorer-item-root
            :class="getCardClass(item)"
            :style="{ height: `${props.cardHeight}px`, marginBottom: `${props.gap}px` }"
            @click="emit('item-click', item)"
            @dblclick="emit('item-open', item)"
            @contextmenu="emit('item-contextmenu', item, $event)"
          >
            <slot name="item" :item="item" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.explorer-grid-card {
  background: color-mix(in oklch, var(--surface-console) 26%, var(--surface-panel));
  border-color: color-mix(in oklch, var(--surface-console) 26%, var(--surface-panel));
  box-shadow: 0 6px 16px -8px rgb(15 23 42 / 0.14);
}

.explorer-grid-card:hover {
  background: color-mix(in oklch, var(--surface-console) 38%, var(--surface-panel));
  border-color: color-mix(in oklch, var(--color-border) 54%, transparent);
  box-shadow: 0 8px 18px -8px rgb(15 23 42 / 0.18);
}

.dark .explorer-grid-card {
  background: color-mix(in oklch, var(--surface-toolbar) 74%, var(--surface-panel));
  border-color: color-mix(in oklch, var(--surface-toolbar) 74%, var(--surface-panel));
  box-shadow: 0 10px 18px -16px rgb(0 0 0 / 0.56);
}

.dark .explorer-grid-card:hover {
  background: color-mix(in oklch, var(--surface-toolbar) 92%, var(--surface-panel));
  border-color: color-mix(in oklch, var(--color-border) 88%, transparent);
  box-shadow: 0 12px 20px -16px rgb(0 0 0 / 0.58);
}

.explorer-grid-card-active,
.explorer-grid-card-active:hover {
  background: color-mix(in oklch, var(--color-primary) 16%, var(--surface-panel));
  border-color: color-mix(in oklch, var(--color-primary) 48%, transparent);
  box-shadow:
    inset 0 0 0 1px color-mix(in oklch, var(--color-primary) 28%, transparent),
    0 8px 18px -10px rgb(15 23 42 / 0.16);
}

.dark .explorer-grid-card-active,
.dark .explorer-grid-card-active:hover {
  background: color-mix(in oklch, var(--color-primary) 12%, var(--surface-toolbar));
  border-color: color-mix(in oklch, var(--color-primary) 42%, transparent);
  box-shadow:
    inset 0 0 0 1px color-mix(in oklch, var(--color-primary) 28%, transparent),
    0 12px 24px -18px rgb(12 18 34 / 0.52);
}
</style>
