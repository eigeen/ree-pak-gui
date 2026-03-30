<script setup lang="ts">
import { computed } from 'vue'
import AppDropdownMenuItems from '@/components/context-menu/AppDropdownMenuItems.vue'
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { compactContextMenuEntries, type ContextMenuEntry } from '@/lib/contextMenu'

const props = withDefaults(
  defineProps<{
    items: ContextMenuEntry[]
    open: boolean
    x: number
    y: number
  }>(),
  {
    open: false,
    x: 0,
    y: 0
  }
)

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void
}>()

const normalizedItems = computed(() => compactContextMenuEntries(props.items))

const modelOpen = computed({
  get: () => props.open && normalizedItems.value.length > 0,
  set: (value: boolean) => emit('update:open', value)
})

const anchorStyle = computed(() => ({
  left: `${props.x}px`,
  top: `${props.y}px`
}))
</script>

<template>
  <DropdownMenu :open="modelOpen" @update:open="modelOpen = $event">
    <DropdownMenuTrigger as-child>
      <button
        type="button"
        tabindex="-1"
        aria-hidden="true"
        class="pointer-events-none fixed h-0 w-0 opacity-0"
        :style="anchorStyle"
      />
    </DropdownMenuTrigger>

    <DropdownMenuContent align="start" side="bottom" :side-offset="2" class="min-w-52">
      <AppDropdownMenuItems :items="normalizedItems" />
    </DropdownMenuContent>
  </DropdownMenu>
</template>
