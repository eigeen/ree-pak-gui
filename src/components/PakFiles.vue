<template>
  <div class="space-y-4">
    <div class="surface-raised min-h-52 border border-border/80 p-3">
      <draggable
        v-model="orderedPakList"
        item-key="path"
        :animation="200"
        class="space-y-2"
        ghost-class="ghost"
        handle=".drag-handle"
        :force-fallback="true"
        @change="onChange"
      >
        <template #item="{ element, index }">
          <AppContextMenu :items="getPakMenuItems(element, index)">
            <PakFileItem :pak-id="element.id" :file-path="element.path" @remove="$emit('close', index)" />
          </AppContextMenu>
        </template>
      </draggable>

      <div v-if="orderedPakList.length === 0" class="empty-state min-h-44">
        <p class="text-base font-semibold text-foreground">{{ t('pakFiles.openPaks') }}</p>
        <p class="section-copy">{{ t('pakFiles.selectFileNameTable') }}</p>
      </div>
    </div>

    <div class="flex gap-3">
      <TooltipProvider v-if="props.showOpenButton">
        <Tooltip>
          <TooltipTrigger as-child>
            <div class="flex-1">
              <Button class="w-full rounded-md" :disabled="!enableAdd" @click="$emit('open')">
                <FolderPlus class="size-4" />
                {{ t('pakFiles.openPaks') }}
              </Button>
            </div>
          </TooltipTrigger>
          <TooltipContent v-if="!enableAdd" class="rounded-xl px-3 py-2 text-sm">
            {{ t('pakFiles.selectFileNameTable') }}
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>

      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger as-child>
            <span>
              <Button
                size="icon"
                variant="outline"
                class="rounded-md"
                :disabled="pakList.length === 0"
                @click="$emit('closeAll')"
              >
                <X class="size-4" />
              </Button>
            </span>
          </TooltipTrigger>
          <TooltipContent v-if="pakList.length > 0" class="rounded-xl px-3 py-2 text-sm">
            {{ t('pakFiles.closeAllPaks') }}
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import draggable from 'vuedraggable'
import { FolderPlus, Info, Trash2, X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { pak_order, type PakId, type PakInfo } from '@/api/tauri/pak'
import AppContextMenu from '@/components/context-menu/AppContextMenu.vue'
import PakFileItem from '@/components/PakFileItem.vue'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import type { ContextMenuEntry } from '@/lib/contextMenu'

const { t } = useI18n()

export interface Props {
  pakList: PakInfo[]
  enableAdd: boolean
  showOpenButton?: boolean
}

export interface OrderedData {
  id: PakId
  path: string
}

const props = withDefaults(defineProps<Props>(), {
  enableAdd: true,
  showOpenButton: true
})

const emit = defineEmits<{
  (e: 'open'): void
  (e: 'close', index: number): void
  (e: 'order', order: PakId[]): void
  (e: 'closeAll'): void
  (e: 'show-properties', pak: OrderedData): void
}>()

const orderedPakList = ref<OrderedData[]>([])

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

  const { oldIndex, newIndex } = event
  if (typeof oldIndex !== 'number' || typeof newIndex !== 'number') {
    return
  }

  const newList = [...orderedPakList.value]
  const item = newList.splice(oldIndex, 1)[0]
  if (!item) {
    return
  }

  newList.splice(newIndex, 0, item)
  const orderList = newList.map((current) => current.id)
  await pak_order(orderList)
  emit('order', orderList)
}

function getPakMenuItems(pak: OrderedData, index: number): ContextMenuEntry[] {
  return [
    {
      type: 'action',
      key: `pak-properties-${pak.id}`,
      label: '查看属性',
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
      label: '移除',
      icon: Trash2,
      destructive: true,
      action: () => emit('close', index)
    }
  ]
}
</script>

<style scoped>
.ghost {
  opacity: 0.45;
}
</style>
