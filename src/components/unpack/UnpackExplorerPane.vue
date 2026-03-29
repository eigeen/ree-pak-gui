<script setup lang="ts">
import {
  ArrowUp,
  Download,
  FileArchive,
  FolderTree,
  LayoutGrid,
  List,
  RefreshCw,
  Search
} from 'lucide-vue-next'
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
  enableExtract: boolean
  hasTree: boolean
  hasPakData: boolean
  showOverlay: boolean
  loadingTree: boolean
  layoutMode: ExplorerLayoutMode
  items: ExplorerEntry[]
  selectedKey: string
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
  (e: 'extract'): void
  (e: 'render'): void
  (e: 'open-directory', id: string): void
  (e: 'open-parent-directory'): void
  (e: 'toggle-layout'): void
  (e: 'item-click', item: ExplorerEntry): void
  (e: 'item-open', item: ExplorerEntry): void
  (e: 'visible-items-change', items: ExplorerEntry[]): void
}>()

const { t } = useI18n()

function handleSearchTextUpdate(value: string | number) {
  emit('update:search-text', String(value))
}
</script>

<template>
  <div class="flex h-full min-w-0 flex-col">
    <div class="desktop-toolbar justify-between">
      <div class="flex min-w-0 flex-1 items-center gap-2 px-2">
        <Search class="size-4 text-muted-foreground" />
        <DenseInput
          :model-value="props.searchText"
          class="w-44 border-border/60 bg-background/80"
          placeholder="Search current folder..."
          @update:model-value="handleSearchTextUpdate"
        />
      </div>
      <Button
        variant="outline"
        size="sm"
        class="desktop-command-button"
        :disabled="!props.enableExtract"
        @click="emit('extract')"
      >
        <Download class="size-4" />
        {{ t('unpack.extract') }}
      </Button>
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

      <div class="flex shrink-0 items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          class="desktop-icon-button h-7 shrink-0 px-2"
          :disabled="!props.hasTree || !props.canGoParentDirectory"
          title="返回上一级目录"
          aria-label="返回上一级目录"
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
        v-if="props.showOverlay"
        class="absolute inset-4 z-20 flex items-center justify-center border border-border/80 bg-background/88 backdrop-blur-sm"
        @click.stop
      >
        <Button :disabled="props.loadingTree" @click="emit('render')">
          <RefreshCw class="size-4" :class="props.loadingTree ? 'animate-spin' : ''" />
          {{ t('unpack.loadFileTree') }}
        </Button>
      </div>

      <div
        class="h-full"
        :class="props.hasPakData && props.hasTree && props.layoutMode === 'details' ? '' : 'p-4'"
      >
        <div v-if="!props.hasPakData" class="empty-state h-full">
          <FileArchive class="size-10 text-muted-foreground" />
          <p class="text-sm font-semibold text-foreground">尚未添加文件</p>
          <p class="section-copy">点击左侧按钮或拖拽文件到窗口中添加 Pak 文件。</p>
        </div>

        <div v-else-if="!props.hasTree" class="empty-state h-full">
          <FolderTree class="size-10 text-muted-foreground" />
          <p class="text-sm font-semibold text-foreground">资源树尚未载入</p>
          <p class="section-copy">选择路径列表后，点击左侧刷新按钮生成 Explorer。</p>
        </div>

        <UnpackExplorerTileView
          v-else-if="props.layoutMode === 'tile'"
          :items="props.items"
          :selected-key="props.selectedKey"
          :reset-key="props.resetKey"
          :texture-preview-enabled="props.texturePreviewEnabled"
          :renderers="props.renderers"
          @item-click="emit('item-click', $event)"
          @item-open="emit('item-open', $event)"
          @visible-items-change="emit('visible-items-change', $event)"
        />

        <UnpackExplorerDetailsView
          v-else
          :items="props.items"
          :selected-key="props.selectedKey"
          :reset-key="props.resetKey"
          :renderers="props.renderers"
          :column-labels="props.columnLabels"
          @item-click="emit('item-click', $event)"
          @item-open="emit('item-open', $event)"
          @visible-items-change="emit('visible-items-change', $event)"
        />
      </div>
    </div>
  </div>
</template>
