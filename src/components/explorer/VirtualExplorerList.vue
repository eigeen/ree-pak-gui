<script setup lang="ts" generic="TItem extends { id: string }">
import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, ref, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    items: TItem[]
    focusedKey?: string
    checkedKeys?: string[]
    resetKey?: string | number
    overscan?: number
    rowHeight?: number
    columnTemplate?: string
  }>(),
  {
    focusedKey: '',
    checkedKeys: () => [],
    resetKey: '',
    overscan: 8,
    rowHeight: 42,
    columnTemplate: 'minmax(0, 1fr)'
  }
)

const emit = defineEmits<{
  (e: 'item-click', item: TItem, event: MouseEvent): void
  (e: 'item-open', item: TItem, event: MouseEvent): void
  (e: 'item-contextmenu', item: TItem, event: MouseEvent): void
  (e: 'item-hover-start', item: TItem, event: PointerEvent): void
  (e: 'item-hover-move', item: TItem, event: PointerEvent): void
  (e: 'item-hover-end', item: TItem, event: PointerEvent): void
  (e: 'background-click', event: MouseEvent): void
  (e: 'background-contextmenu', event: MouseEvent): void
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
const checkedKeySet = computed(() => new Set(props.checkedKeys))

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
  const classNames = ['explorer-list-row absolute left-0 top-0 w-full px-3 text-left']

  if (checkedKeySet.value.has(item.id)) {
    classNames.push('explorer-list-row-checked border-b border-transparent text-foreground')
  } else {
    classNames.push('border-b border-border/45 text-foreground')
  }

  return classNames.join(' ')
}

function handleBackgroundClick(event: MouseEvent) {
  const target = event.target
  if (target instanceof Element && target.closest('[data-explorer-item-root]')) {
    return
  }

  emit('background-click', event)
}

function handleBackgroundContextMenu(event: MouseEvent) {
  const target = event.target
  if (target instanceof Element && target.closest('[data-explorer-item-root]')) {
    return
  }

  event.preventDefault()
  emit('background-contextmenu', event)
}
</script>

<template>
  <div class="explorer-list-shell flex h-full min-h-0 flex-col overflow-hidden">
    <div
      class="explorer-list-header text-2xs grid shrink-0 items-center gap-3 border-b border-border/70 px-3 py-2 font-semibold tracking-[0.12em] text-muted-foreground/85 uppercase"
      :style="columnStyle"
    >
      <slot name="header" />
    </div>

    <div
      ref="scrollElementRef"
      class="editor-scrollbar min-h-0 flex-1 overflow-auto"
      @click="handleBackgroundClick"
      @contextmenu="handleBackgroundContextMenu"
    >
      <div class="relative min-h-full" :style="{ height: `${totalSize}px` }">
        <button
          v-for="virtualRow in virtualRows"
          :key="String(virtualRow.key)"
          type="button"
          data-explorer-item-root
          :class="getRowClass(getItem(virtualRow.index))"
          :style="getRowStyle(virtualRow.start)"
          @click="emit('item-click', getItem(virtualRow.index), $event)"
          @dblclick="emit('item-open', getItem(virtualRow.index), $event)"
          @contextmenu.prevent="emit('item-contextmenu', getItem(virtualRow.index), $event)"
          @pointerenter="emit('item-hover-start', getItem(virtualRow.index), $event)"
          @pointermove="emit('item-hover-move', getItem(virtualRow.index), $event)"
          @pointerleave="emit('item-hover-end', getItem(virtualRow.index), $event)"
        >
          <div class="grid h-full items-center gap-3" :style="columnStyle">
            <slot name="row" :item="getItem(virtualRow.index)" />
          </div>
        </button>
      </div>
    </div>
  </div>
</template>
