<script setup lang="ts">
import { Copy, MoreHorizontal } from 'lucide-vue-next'
import {
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger
} from '@/components/ui/context-menu'
import type { ContextMenuEntry } from '@/lib/contextMenu'

defineOptions({
  name: 'AppContextMenuItems'
})

defineProps<{
  items: ContextMenuEntry[]
}>()
</script>

<template>
  <template v-for="entry in items" :key="entry.key">
    <ContextMenuSeparator v-if="entry.type === 'separator'" />

    <ContextMenuSub v-else-if="entry.type === 'submenu'">
      <ContextMenuSubTrigger :disabled="entry.disabled">
        <component :is="entry.icon ?? MoreHorizontal" class="size-4" />
        <span>{{ entry.label }}</span>
      </ContextMenuSubTrigger>
      <ContextMenuSubContent class="min-w-52">
        <AppContextMenuItems :items="entry.children" />
      </ContextMenuSubContent>
    </ContextMenuSub>

    <ContextMenuItem
      v-else
      :disabled="entry.disabled"
      :variant="entry.destructive ? 'destructive' : 'default'"
      @select="entry.action"
    >
      <component :is="entry.icon ?? Copy" class="size-4" />
      <span>{{ entry.label }}</span>
      <ContextMenuShortcut v-if="entry.shortcut">
        {{ entry.shortcut }}
      </ContextMenuShortcut>
    </ContextMenuItem>
  </template>
</template>
