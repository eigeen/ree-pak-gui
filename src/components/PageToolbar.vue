<template>
  <div class="desktop-window-toolbar">
    <div class="flex min-w-0 items-center gap-0">
      <DesktopMenuBar class="min-w-0" :items="items" />
    </div>

    <div class="flex-1" />

    <div v-if="$slots.right" class="flex min-w-0 items-center gap-2">
      <slot name="right" />
    </div>
  </div>
</template>

<script setup lang="ts">
import DesktopMenuBar, { type MenuGroup } from '@/components/DesktopMenuBar.vue'

defineProps<{
  items: MenuGroup[]
}>()
</script>
