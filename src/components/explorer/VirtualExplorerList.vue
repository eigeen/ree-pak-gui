<script setup lang="ts" generic="TItem extends { id: string }">
import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, ref, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    items: TItem[]
    selectedKey?: string
    resetKey?: string | number
    overscan?: number
    rowHeight?: number
    columnTemplate?: string
  }>(),
  {
    selectedKey: '',
    resetKey: '',
    overscan: 8,
    rowHeight: 42,
    columnTemplate: 'minmax(0, 1fr)'
  }
)

const emit = defineEmits<{
  (e: 'item-click', item: TItem): void
  (e: 'item-open', item: TItem): void
  (e: 'visible-items-change', items: TItem[]): void
}>()

const scrollElementRef = ref<HTMLElement | null>(null)

const virtualizer = useVirtualizer<HTMLElement, HTMLDivElement>(
  computed(() => ({
    count: props.items.length,
    getScrollElement: () => scrollElementRef.value,
    estimateSize: () => props.rowHeight,
    overscan: props.overscan,
    getItemKey: (index: number) => props.items[index]?.id ?? `explorer-item-${index}`
  }))
)

const virtualRows = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
const visibleItems = computed(() =>
  virtualRows.value
    .map((virtualRow) => props.items[virtualRow.index])
    .filter((item): item is TItem => Boolean(item))
)
const columnStyle = computed(() => ({
  gridTemplateColumns: props.columnTemplate
}))

watch(
  visibleItems,
  (items) => {
    emit('visible-items-change', items)
  },
  { immediate: true }
)

watch(
  () => props.resetKey,
  () => {
    scrollElementRef.value?.scrollTo({ top: 0, behavior: 'auto' })
    virtualizer.value.scrollToOffset(0)
  }
)

watch(
  () => props.items.length,
  () => {
    virtualizer.value.measure()
  }
)

function getItem(index: number): TItem {
  const item = props.items[index]
  if (!item) {
    throw new Error(`VirtualExplorerList item not found at index ${index}`)
  }

  return item
}

function getRowStyle(start: number) {
  return {
    height: `${props.rowHeight}px`,
    transform: `translateY(${start}px)`
  }
}

function getRowClass(item: TItem) {
  const baseClass =
    'absolute left-0 top-0 w-full px-3 text-left transition-colors duration-150 hover:bg-[#1d2028]'
  if (props.selectedKey !== item.id) {
    return `${baseClass} border-b border-border/45 text-foreground`
  }

  return `${baseClass} z-10 border-b border-transparent bg-[#202838] text-foreground shadow-[inset_0_0_0_1px_rgba(122,162,255,0.28)]`
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col overflow-hidden bg-[#101117]">
    <div
      class="grid shrink-0 items-center gap-3 border-b border-border/70 bg-[#151822] px-3 py-2 text-[11px] font-semibold tracking-[0.12em] text-muted-foreground/85 uppercase"
      :style="columnStyle"
    >
      <slot name="header" />
    </div>

    <div ref="scrollElementRef" class="editor-scrollbar min-h-0 flex-1 overflow-auto">
      <div class="relative min-h-full" :style="{ height: `${totalSize}px` }">
        <button
          v-for="virtualRow in virtualRows"
          :key="String(virtualRow.key)"
          type="button"
          :class="getRowClass(getItem(virtualRow.index))"
          :style="getRowStyle(virtualRow.start)"
          @click="emit('item-click', getItem(virtualRow.index))"
          @dblclick="emit('item-open', getItem(virtualRow.index))"
        >
          <div class="grid h-full items-center gap-3" :style="columnStyle">
            <slot name="row" :item="getItem(virtualRow.index)" />
          </div>
        </button>
      </div>
    </div>
  </div>
</template>
