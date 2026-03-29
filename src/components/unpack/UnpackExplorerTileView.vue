<script setup lang="ts">
import { File, Folder } from 'lucide-vue-next'
import VirtualExplorerGrid from '@/components/explorer/VirtualExplorerGrid.vue'
import type { ExplorerEntry, ExplorerRenderers } from '@/lib/unpackExplorer'

const props = defineProps<{
  items: ExplorerEntry[]
  selectedKey: string
  resetKey: string | number
  texturePreviewEnabled: boolean
  renderers: ExplorerRenderers
}>()

const emit = defineEmits<{
  (e: 'item-click', item: ExplorerEntry): void
  (e: 'item-open', item: ExplorerEntry): void
  (e: 'visible-items-change', items: ExplorerEntry[]): void
}>()
</script>

<template>
  <VirtualExplorerGrid
    :items="props.items"
    :selected-key="props.selectedKey"
    :reset-key="props.resetKey"
    class="h-full"
    @item-click="emit('item-click', $event)"
    @item-open="emit('item-open', $event)"
    @visible-items-change="emit('visible-items-change', $event)"
  >
    <template #item="{ item }">
      <div class="flex h-full min-h-0 flex-col">
        <div
          :class="[
            'explorer-grid-preview relative flex h-30 overflow-hidden',
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
              :alt="item.name"
              fit="cover"
              class="asset-tile-preview"
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
            class="text-ui-xs line-clamp-2 min-h-[2.5rem] break-all font-semibold leading-5 text-foreground"
          >
            {{ item.name }}
          </p>

          <div
            class="text-ui-tiny mt-auto flex items-center justify-between gap-3 pt-2 text-muted-foreground"
          >
            <span class="truncate">{{ props.renderers.getItemTypeLabel(item) }}</span>
            <template v-if="item.isDir">
              <span class="asset-counts shrink-0">
                <span class="asset-count-chip">
                  <Folder class="size-3" />
                  {{ props.renderers.getDirectoryCounts(item).folders }}
                </span>
                <span class="asset-count-chip">
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
.asset-tile-preview {
  height: 100%;
  width: 100%;
}

.asset-hero-icon {
  filter: drop-shadow(0 10px 18px rgb(0 0 0 / 0.28));
}

.asset-counts {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
}

.asset-count-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
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
