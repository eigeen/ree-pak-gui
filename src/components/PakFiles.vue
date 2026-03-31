<template>
  <div class="flex h-full min-h-0 flex-col">
    <div class="flex min-h-52 flex-1 flex-col overflow-hidden p-0 border-t">
      <div class="flex items-center justify-end px-2 py-1">
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger as-child>
              <span>
                <Button size="icon-sm" variant="ghost" class="rounded-full" @click="$emit('open')">
                  <FolderPlus class="size-4" />
                </Button>
              </span>
            </TooltipTrigger>
            <TooltipContent class="rounded-xl px-3 py-2 text-sm">
              {{ t('pakFiles.openPaks') }}
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>

        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger as-child>
              <span>
                <Button
                  size="icon-sm"
                  variant="ghost"
                  class="rounded-full"
                  :disabled="pakList.length === 0"
                  @click="$emit('closeAll')"
                >
                  <X class="size-4" />
                </Button>
              </span>
            </TooltipTrigger>
            <TooltipContent class="rounded-xl px-3 py-2 text-sm">
              {{ t('pakFiles.closeAllPaks') }}
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </div>

      <draggable
        v-if="orderedPakList.length > 0"
        v-model="orderedPakList"
        item-key="path"
        :animation="200"
        class="editor-scrollbar flex h-full min-h-0 flex-col gap-0.5 overflow-auto rounded-[1.05rem]"
        ghost-class="ghost"
        handle=".drag-handle"
        :force-fallback="true"
        @change="onChange"
      >
        <template #item="{ element, index }">
          <PakFileItem
            :pak-id="element.id"
            :file-path="element.path"
            @remove="$emit('close', index)"
            @contextmenu="openPakContextMenu($event, element, index)"
          />
        </template>
      </draggable>

      <div v-else class="empty-state min-h-full border-border/70 bg-background/45">
        <p class="text-base font-semibold text-foreground">{{ t('pakFiles.openPaks') }}</p>
        <p class="section-copy">{{ t('pakFiles.emptyHint') }}</p>
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="pakContextMenu.open"
        class="fixed inset-0 z-50"
        @contextmenu.prevent="closePakContextMenu"
        @pointerdown="closePakContextMenu"
      >
        <div
          class="bg-popover text-popover-foreground absolute min-w-52 overflow-hidden rounded-md border border-border/80 p-1 shadow-md"
          :style="{
            left: `${pakContextMenu.x}px`,
            top: `${pakContextMenu.y}px`
          }"
          @pointerdown.stop
        >
          <template v-for="entry in pakContextMenu.items" :key="entry.key">
            <div v-if="entry.type === 'separator'" class="my-1 h-px bg-border/80" />
            <button
              v-else-if="entry.type === 'action'"
              type="button"
              class="focus:bg-accent focus:text-accent-foreground flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-left text-sm outline-hidden disabled:pointer-events-none disabled:opacity-50"
              :class="entry.destructive ? 'text-destructive' : ''"
              :disabled="entry.disabled"
              @click="runPakContextAction(entry.action)"
            >
              <component :is="entry.icon ?? MoreHorizontal" class="size-4 shrink-0" />
              <span>{{ entry.label }}</span>
            </button>
          </template>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import draggable from 'vuedraggable'
import { FolderPlus, Info, MoreHorizontal, Trash2, X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { pak_order, type PakId, type PakInfo } from '@/api/tauri/pak'
import PakFileItem from '@/components/PakFileItem.vue'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import {
  compactContextMenuEntries,
  type ContextMenuActionItem,
  type ContextMenuEntry
} from '@/lib/contextMenu'

const { t } = useI18n()

export interface Props {
  pakList: PakInfo[]
}

export interface OrderedData {
  id: PakId
  path: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'open'): void
  (e: 'close', index: number): void
  (e: 'order', order: PakId[]): void
  (e: 'closeAll'): void
  (e: 'show-properties', pak: OrderedData): void
}>()

const orderedPakList = ref<OrderedData[]>([])
const pakContextMenu = ref<{
  open: boolean
  x: number
  y: number
  items: ContextMenuEntry[]
}>({
  open: false,
  x: 0,
  y: 0,
  items: []
})

watch(
  () => props.pakList,
  (newValue: PakInfo[]) => {
    orderedPakList.value = newValue.map((pak) => ({
      id: pak.id,
      path: pak.path
    }))
  },
  { immediate: true }
)

async function onChange(event: any) {
  if (!event.moved) {
    return
  }

  const orderList = orderedPakList.value.map((current) => current.id)
  await pak_order(orderList)
  emit('order', orderList)
}

function getPakMenuItems(pak: OrderedData, index: number): ContextMenuEntry[] {
  return [
    {
      type: 'action',
      key: `pak-properties-${pak.id}`,
      label: t('pakFiles.viewProperties'),
      icon: Info,
      action: () => emit('show-properties', pak)
    },
    {
      type: 'separator',
      key: `pak-separator-${pak.id}`
    },
    {
      type: 'action',
      key: `pak-remove-${pak.id}`,
      label: t('pakFiles.remove'),
      icon: Trash2,
      destructive: true,
      action: () => emit('close', index)
    }
  ]
}

function openPakContextMenu(event: MouseEvent, pak: OrderedData, index: number) {
  event.preventDefault()
  event.stopPropagation()
  pakContextMenu.value = {
    open: true,
    x: event.clientX,
    y: event.clientY,
    items: compactContextMenuEntries(getPakMenuItems(pak, index))
  }
}

function closePakContextMenu() {
  if (!pakContextMenu.value.open) {
    return
  }

  pakContextMenu.value = {
    open: false,
    x: 0,
    y: 0,
    items: []
  }
}

async function runPakContextAction(action: ContextMenuActionItem['action']) {
  closePakContextMenu()
  await action()
}

function handleWindowKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    closePakContextMenu()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleWindowKeydown)
  window.addEventListener('blur', closePakContextMenu)
  window.addEventListener('resize', closePakContextMenu)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleWindowKeydown)
  window.removeEventListener('blur', closePakContextMenu)
  window.removeEventListener('resize', closePakContextMenu)
})
</script>

<style scoped>
.ghost {
  opacity: 0.55;
  filter: saturate(0.9);
}
</style>
