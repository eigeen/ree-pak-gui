<script setup lang="ts">
import { computed } from 'vue'
import AppContextMenuItems from '@/components/context-menu/AppContextMenuItems.vue'
import { ContextMenu, ContextMenuContent, ContextMenuTrigger } from '@/components/ui/context-menu'
import { compactContextMenuEntries, type ContextMenuEntry } from '@/lib/contextMenu'

const props = withDefaults(
  defineProps<{
    items: ContextMenuEntry[]
    disabled?: boolean
  }>(),
  {
    disabled: false
  }
)

const normalizedItems = computed(() => compactContextMenuEntries(props.items))
const triggerDisabled = computed(() => props.disabled || normalizedItems.value.length === 0)
</script>

<template>
  <ContextMenu>
    <ContextMenuTrigger :disabled="triggerDisabled">
      <div class="h-full min-h-0">
        <slot />
      </div>
    </ContextMenuTrigger>

    <ContextMenuContent class="min-w-52">
      <AppContextMenuItems :items="normalizedItems" />
    </ContextMenuContent>
  </ContextMenu>
</template>
