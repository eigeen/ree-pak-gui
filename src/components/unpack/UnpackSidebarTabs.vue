<template>
  <div class="desktop-toolbar">
    <DesktopTabs v-model="selectedValue" :tabs="props.tabs" />
  </div>
</template>

<script setup lang="ts">
import type { DesktopTabItem } from '@/components/DesktopTabs.vue'
import DesktopTabs from '@/components/DesktopTabs.vue'

export interface UnpackSidebarTabItem extends DesktopTabItem {
  icon: NonNullable<DesktopTabItem['icon']>
}

interface Props {
  tabs: UnpackSidebarTabItem[]
}

const selectedValue = defineModel<string>({ required: true })
const props = defineProps<Props>()
</script>
