<template>
  <template v-for="item in props.items" :key="item.value">
    <component
      :is="item.component"
      v-if="shouldRender(item)"
      v-show="item.unmountInactive || props.activeValue === item.value"
    />
  </template>
</template>

<script setup lang="ts">
import { ref, watch, type Component } from 'vue'

export interface DesktopTabPanelItem {
  value: string
  component: Component
  unmountInactive?: boolean
}

interface Props {
  activeValue: string
  items: DesktopTabPanelItem[]
}

const props = defineProps<Props>()
const mountedValues = ref(new Set<string>())

watch(
  () => props.activeValue,
  (value) => {
    mountedValues.value.add(value)
  },
  { immediate: true }
)

function shouldRender(item: DesktopTabPanelItem) {
  if (item.unmountInactive) {
    return props.activeValue === item.value
  }

  return mountedValues.value.has(item.value)
}
</script>
