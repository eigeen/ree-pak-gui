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
  const baseClass = 'relative isolate group flex min-h-0 flex-col overflow-hidden rounded-[0.4rem] bg-[#22242c] text-left shadow-[0_14px_28px_-24px_rgba(0,0,0,0.95)] transition-[background-color,box-shadow] duration-150 hover:bg-[#272a33] hover:shadow-[0_18px_30px_-24px_rgba(0,0,0,1)]'
  if (props.selectedKey !== item.id) {
    return `${baseClass} ring-1 ring-transparent`
  }

  return `${baseClass} bg-[#2a2d37] ring-1 ring-[#8ba5ff]/45 shadow-[0_0_0_1px_rgba(139,165,255,0.18),0_18px_40px_-28px_rgba(24,48,102,0.92)]`
}
</script>

<template>
  <div ref="scrollElementRef" class="editor-scrollbar h-full overflow-auto pr-1">
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
            :class="getCardClass(item)"
            :style="{ height: `${props.cardHeight}px`, marginBottom: `${props.gap}px` }"
            @click="emit('item-click', item)"
            @dblclick="emit('item-open', item)"
          >
            <slot name="item" :item="item" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
