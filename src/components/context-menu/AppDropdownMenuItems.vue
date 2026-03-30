<script setup lang="ts">
import { Copy, MoreHorizontal } from 'lucide-vue-next'
import {
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger
} from '@/components/ui/dropdown-menu'
import type { ContextMenuEntry } from '@/lib/contextMenu'

defineOptions({
  name: 'AppDropdownMenuItems'
})

defineProps<{
  items: ContextMenuEntry[]
}>()
</script>

<template>
  <template v-for="entry in items" :key="entry.key">
    <DropdownMenuSeparator v-if="entry.type === 'separator'" />

    <DropdownMenuSub v-else-if="entry.type === 'submenu'">
      <DropdownMenuSubTrigger :disabled="entry.disabled">
        <component :is="entry.icon ?? MoreHorizontal" class="size-4" />
        <span>{{ entry.label }}</span>
      </DropdownMenuSubTrigger>
      <DropdownMenuSubContent class="min-w-52">
        <AppDropdownMenuItems :items="entry.children" />
      </DropdownMenuSubContent>
    </DropdownMenuSub>

    <DropdownMenuItem
      v-else
      :disabled="entry.disabled"
      :variant="entry.destructive ? 'destructive' : 'default'"
      @select="entry.action"
    >
      <component :is="entry.icon ?? Copy" class="size-4" />
      <span>{{ entry.label }}</span>
      <DropdownMenuShortcut v-if="entry.shortcut">
        {{ entry.shortcut }}
      </DropdownMenuShortcut>
    </DropdownMenuItem>
  </template>
</template>
