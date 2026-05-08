<script setup lang="ts">
import { File, Folder } from 'lucide-vue-next'
import { ElImage } from 'element-plus'
import VirtualExplorerGrid from '@/components/explorer/VirtualExplorerGrid.vue'
import type { ExplorerEntry, ExplorerRenderers } from '@/lib/unpackExplorer'

const props = defineProps<{
  items: ExplorerEntry[]
  focusedKey: string
  checkedKeys: string[]
  resetKey: string | number
  texturePreviewEnabled: boolean
  renderers: ExplorerRenderers
}>()

const emit = defineEmits<{
  (e: 'item-click', item: ExplorerEntry, event: MouseEvent): void
  (e: 'item-open', item: ExplorerEntry, event: MouseEvent): void
  (e: 'item-contextmenu', item: ExplorerEntry, event: MouseEvent): void
  (e: 'background-click', event: MouseEvent): void
  (e: 'background-contextmenu', event: MouseEvent): void
  (e: 'visible-items-change', items: ExplorerEntry[]): void
}>()

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
  <VirtualExplorerGrid
    :items="props.items"
    :focused-key="props.focusedKey"
    :checked-keys="props.checkedKeys"
    :reset-key="props.resetKey"
    class="h-full"
    @item-click="handleItemClick"
    @item-open="handleItemOpen"
    @item-contextmenu="handleItemContextMenu"
    @background-click="emit('background-click', $event)"
    @background-contextmenu="emit('background-contextmenu', $event)"
    @visible-items-change="emit('visible-items-change', $event)"
  >
    <template #item="{ item }">
      <div class="flex h-full min-h-0 flex-col">
        <div
          :class="[
            'relative flex h-30 overflow-hidden bg-[color-mix(in_oklch,var(--surface-console)_18%,var(--surface-toolbar))] dark:bg-[color-mix(in_oklch,var(--surface-toolbar)_86%,var(--surface-panel))]',
            props.texturePreviewEnabled && props.renderers.getTexturePreview(item)
              ? 'items-stretch justify-stretch'
              : 'items-center justify-center px-3 py-3'
          ]"
          :style="props.renderers.getPreviewSurfaceStyle(item)"
        >
          <template v-if="item.isDir">
            <component
              :is="props.renderers.getHeroIcon(item)"
              class="asset-hero-icon size-14"
              :style="props.renderers.getHeroIconStyle(item)"
            />
          </template>
          <template
            v-else-if="props.texturePreviewEnabled && props.renderers.getTexturePreview(item)"
          >
            <el-image
              :src="props.renderers.getTexturePreview(item) ?? undefined"
              :alt="item.displayName ?? item.name"
              fit="cover"
              class="asset-tile-preview size-full"
            >
              <template #error>
                <div class="flex h-full w-full items-center justify-center">
                  <component
                    :is="props.renderers.getHeroIcon(item)"
                    class="asset-hero-icon size-12"
                    :style="props.renderers.getHeroIconStyle(item)"
                  />
                </div>
              </template>
            </el-image>
          </template>
          <template v-else>
            <component
              :is="props.renderers.getHeroIcon(item)"
              class="asset-hero-icon size-12"
              :style="props.renderers.getHeroIconStyle(item)"
            />
          </template>
        </div>

        <!-- split line -->
        <div class="h-0.5 shrink-0" :style="props.renderers.getAccentStyle(item)" />

        <div class="flex min-h-0 flex-1 flex-col px-3 py-2.5">
          <p
            class="text-sm line-clamp-2 min-h-[2.5rem] break-all font-semibold leading-5 text-foreground"
          >
            {{ item.displayName ?? item.name }}
          </p>

          <div v-if="item.isDir" class="pt-1 text-2xs text-muted-foreground">
            {{ item.sizeText }}
          </div>

          <div
            class="text-xs mt-auto flex items-center justify-between gap-3 pt-2 text-muted-foreground"
          >
            <span class="truncate">{{ props.renderers.getItemTypeLabel(item) }}</span>
            <template v-if="item.isDir">
              <span class="inline-flex shrink-0 items-center gap-1.5">
                <span class="asset-count-chip inline-flex items-center gap-1">
                  <Folder class="size-3" />
                  {{ props.renderers.getDirectoryCounts(item).folders }}
                </span>
                <span class="asset-count-chip inline-flex items-center gap-1">
                  <File class="size-3" />
                  {{ props.renderers.getDirectoryCounts(item).files }}
                </span>
              </span>
            </template>
            <span v-else class="shrink-0">{{ item.sizeText }}</span>
          </div>
        </div>
      </div>
    </template>
  </VirtualExplorerGrid>
</template>

<style scoped>
.asset-hero-icon {
  filter: drop-shadow(0 10px 18px rgb(0 0 0 / 0.28));
}

.asset-tile-preview :deep(.el-image__wrapper),
.asset-tile-preview :deep(.el-image__inner) {
  height: 100%;
  width: 100%;
}

.asset-tile-preview :deep(.el-image__inner) {
  object-fit: cover;
}

.asset-tile-preview :deep(.el-image__error) {
  background: transparent;
}
</style>
