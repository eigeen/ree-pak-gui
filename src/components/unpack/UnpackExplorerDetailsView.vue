<script setup lang="ts">
import { Checkbox } from '@/components/ui/checkbox'
import VirtualExplorerList from '@/components/explorer/VirtualExplorerList.vue'
import type { ExplorerColumnLabels, ExplorerEntry, ExplorerRenderers } from '@/lib/unpackExplorer'

const props = defineProps<{
  items: ExplorerEntry[]
  focusedKey: string
  checkedKeys: string[]
  resetKey: string | number
  renderers: ExplorerRenderers
  columnLabels: ExplorerColumnLabels
}>()

const emit = defineEmits<{
  (e: 'item-click', item: ExplorerEntry, event: MouseEvent): void
  (e: 'item-open', item: ExplorerEntry, event: MouseEvent): void
  (e: 'item-contextmenu', item: ExplorerEntry, event: MouseEvent): void
  (e: 'item-check', item: ExplorerEntry, checked: boolean): void
  (e: 'background-click', event: MouseEvent): void
  (e: 'background-contextmenu', event: MouseEvent): void
  (e: 'visible-items-change', items: ExplorerEntry[]): void
}>()

const columnTemplate =
  '30px minmax(0, 2.3fr) minmax(110px, 0.9fr) minmax(96px, 0.72fr) minmax(160px, 1.15fr)'

function handleItemContextMenu(item: ExplorerEntry, event: MouseEvent) {
  emit('item-contextmenu', item, event)
}

function handleItemClick(item: ExplorerEntry, event: MouseEvent) {
  emit('item-click', item, event)
}

function handleItemOpen(item: ExplorerEntry, event: MouseEvent) {
  emit('item-open', item, event)
}
</script>

<template>
  <VirtualExplorerList
    :items="props.items"
    :focused-key="props.focusedKey"
    :checked-keys="props.checkedKeys"
    :reset-key="props.resetKey"
    :column-template="columnTemplate"
    :row-height="36"
    class="h-full"
    @item-click="handleItemClick"
    @item-open="handleItemOpen"
    @item-contextmenu="handleItemContextMenu"
    @background-click="emit('background-click', $event)"
    @background-contextmenu="emit('background-contextmenu', $event)"
    @visible-items-change="emit('visible-items-change', $event)"
  >
    <template #header>
      <span />
      <span class="truncate">{{ props.columnLabels.name }}</span>
      <span class="truncate">{{ props.columnLabels.type }}</span>
      <span class="truncate text-right">{{ props.columnLabels.size }}</span>
      <span class="truncate">{{ props.columnLabels.details }}</span>
    </template>

    <template #row="{ item }">
      <div class="flex items-center justify-center">
        <Checkbox
          :model-value="props.checkedKeys.includes(item.id)"
          :aria-label="item.name"
          @update:model-value="emit('item-check', item, $event === true)"
          @click.stop
        />
      </div>

      <div class="flex min-w-0 items-center gap-2.5">
        <component
          :is="props.renderers.getHeroIcon(item)"
          class="asset-list-icon size-4"
          :style="props.renderers.getHeroIconStyle(item)"
        />

        <span class="text-sm truncate font-medium text-foreground">
          {{ item.name }}
        </span>
      </div>

      <span class="text-2xs truncate text-muted-foreground">
        {{ props.renderers.getItemTypeLabel(item) }}
      </span>

      <span class="text-2xs truncate text-right text-muted-foreground">
        {{ item.isDir ? '—' : item.sizeText }}
      </span>

      <span class="text-2xs truncate text-muted-foreground">
        {{ props.renderers.getDetailText(item) }}
      </span>
    </template>
  </VirtualExplorerList>
</template>

<style scoped>
.asset-list-icon {
  filter: drop-shadow(0 4px 8px rgb(0 0 0 / 0.2));
}
</style>
