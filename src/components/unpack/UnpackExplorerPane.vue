<script setup lang="ts">
import { ArrowUp, FileArchive, FolderTree, LayoutGrid, List, Search } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import UnpackExplorerDetailsView from '@/components/unpack/UnpackExplorerDetailsView.vue'
import UnpackExplorerTileView from '@/components/unpack/UnpackExplorerTileView.vue'
import { Button } from '@/components/ui/button'
import { DenseInput } from '@/components/ui/input'
import type {
  ExplorerBreadcrumbSegment,
  ExplorerColumnLabels,
  ExplorerEntry,
  ExplorerLayoutMode,
  ExplorerRenderers
} from '@/lib/unpackExplorer'

const props = defineProps<{
  searchText: string
  hasTree: boolean
  hasPakData: boolean
  layoutMode: ExplorerLayoutMode
  items: ExplorerEntry[]
  focusedKey: string
  checkedKeys: string[]
  resetKey: string | number
  breadcrumbSegments: ExplorerBreadcrumbSegment[]
  currentDirectoryKey: string
  canGoParentDirectory: boolean
  texturePreviewEnabled: boolean
  renderers: ExplorerRenderers
  columnLabels: ExplorerColumnLabels
}>()

const emit = defineEmits<{
  (e: 'update:search-text', value: string): void
  (e: 'open-directory', id: string): void
  (e: 'open-parent-directory'): void
  (e: 'toggle-layout'): void
  (e: 'item-click', item: ExplorerEntry, event: MouseEvent): void
  (e: 'item-check', item: ExplorerEntry, checked: boolean): void
  (e: 'item-open', item: ExplorerEntry, event: MouseEvent): void
  (e: 'item-contextmenu', item: ExplorerEntry, event: MouseEvent): void
  (e: 'background-click', event: MouseEvent): void
  (e: 'background-contextmenu', event: MouseEvent): void
  (e: 'visible-items-change', items: ExplorerEntry[]): void
}>()

const { t } = useI18n()

function handleSearchTextUpdate(value: string | number) {
  emit('update:search-text', String(value))
}

function handleItemContextMenu(item: ExplorerEntry, event: MouseEvent) {
  emit('item-contextmenu', item, event)
}

function handleItemClick(item: ExplorerEntry, event: MouseEvent) {
  emit('item-click', item, event)
}

function handleItemCheck(item: ExplorerEntry, checked: boolean) {
  emit('item-check', item, checked)
}

function handleItemOpen(item: ExplorerEntry, event: MouseEvent) {
  emit('item-open', item, event)
}
</script>

<template>
  <div class="flex h-full min-w-0 flex-col">
    <div class="desktop-toolbar">
      <div class="flex min-w-0 flex-1 items-center gap-2 px-2 py-0.5">
        <Search class="size-4 text-muted-foreground" />
        <DenseInput
          :model-value="props.searchText"
          class="w-44 border-border/60 bg-background/80"
          :placeholder="t('explorer.searchCurrentFolder')"
          @update:model-value="handleSearchTextUpdate"
        />
      </div>
    </div>

    <div class="desktop-subtoolbar items-center justify-between gap-3">
      <div class="flex min-w-0 flex-1 items-center overflow-hidden">
        <div
          v-for="(segment, index) in props.breadcrumbSegments"
          :key="`${segment.id}-${index}-${segment.label}`"
          class="flex min-w-0 items-center"
        >
          <span v-if="index > 0" class="px-1 text-muted-foreground/80">/</span>
          <button
            type="button"
            class="truncate transition-colors hover:text-foreground"
            :class="segment.id === props.currentDirectoryKey ? 'font-medium text-foreground' : ''"
            @click="emit('open-directory', segment.id)"
          >
            {{ segment.label }}
          </button>
        </div>
      </div>

      <div class="flex shrink-0 items-center">
        <Button
          variant="ghost"
          size="sm"
          class="desktop-icon-button h-7 shrink-0 px-2"
          :disabled="!props.hasTree || !props.canGoParentDirectory"
          :title="t('unpack.openParentDirectory')"
          :aria-label="t('unpack.openParentDirectory')"
          @click="emit('open-parent-directory')"
        >
          <ArrowUp class="size-4" />
        </Button>

        <Button
          variant="ghost"
          size="sm"
          class="desktop-icon-button h-7 shrink-0 gap-1.5 px-2 text-xs"
          :disabled="!props.hasTree"
          :title="t('unpack.switchLayout')"
          :aria-label="t('unpack.switchLayout')"
          @click="emit('toggle-layout')"
        >
          <component :is="props.layoutMode === 'tile' ? LayoutGrid : List" class="size-4" />
          {{ props.layoutMode === 'tile' ? t('unpack.layoutTile') : t('unpack.layoutDetails') }}
        </Button>
      </div>
    </div>

    <div class="relative min-h-0 flex-1">
      <div
        class="h-full"
        :class="props.hasPakData && props.hasTree && props.layoutMode === 'details' ? '' : 'p-4'"
      >
        <div v-if="!props.hasPakData" class="empty-state h-full">
          <FileArchive class="size-10 text-muted-foreground" />
          <p class="text-lg font-semibold text-foreground">{{ t('explorer.emptyNoFiles') }}</p>
          <p class="text-sm text-muted-foreground">{{ t('explorer.emptyNoFilesDescription') }}</p>
        </div>

        <div v-else-if="!props.hasTree" class="empty-state h-full">
          <FolderTree class="size-10 text-muted-foreground" />
          <p class="text-lg font-semibold text-foreground">{{ t('explorer.emptyNoTree') }}</p>
          <p class="text-sm text-muted-foreground">{{ t('explorer.emptyNoTreeDescription') }}</p>
        </div>

        <UnpackExplorerTileView
          v-else-if="props.layoutMode === 'tile'"
          :items="props.items"
          :focused-key="props.focusedKey"
          :checked-keys="props.checkedKeys"
          :reset-key="props.resetKey"
          :texture-preview-enabled="props.texturePreviewEnabled"
          :renderers="props.renderers"
          @item-click="handleItemClick"
          @item-open="handleItemOpen"
          @item-contextmenu="handleItemContextMenu"
          @background-click="emit('background-click', $event)"
          @background-contextmenu="emit('background-contextmenu', $event)"
          @visible-items-change="emit('visible-items-change', $event)"
        />

        <UnpackExplorerDetailsView
          v-else
          :items="props.items"
          :focused-key="props.focusedKey"
          :checked-keys="props.checkedKeys"
          :reset-key="props.resetKey"
          :renderers="props.renderers"
          :column-labels="props.columnLabels"
          @item-click="handleItemClick"
          @item-check="handleItemCheck"
          @item-open="handleItemOpen"
          @item-contextmenu="handleItemContextMenu"
          @background-click="emit('background-click', $event)"
          @background-contextmenu="emit('background-contextmenu', $event)"
          @visible-items-change="emit('visible-items-change', $event)"
        />
      </div>
    </div>
  </div>
</template>
