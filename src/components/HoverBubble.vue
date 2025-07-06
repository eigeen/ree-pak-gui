<template>
  <div
    class="inline-flex items-center justify-center relative transition-opacity duration-200"
    :style="{ opacity: opacity }"
  >
    <v-icon icon="mdi-help-circle" :size="size" :color="color" />
    <v-tooltip activator="parent" location="top">
      <slot />
    </v-tooltip>
  </div>
</template>

<script setup lang="ts">
interface Props {
  color?: string
  size?: string | number
  opacity?: number
}

withDefaults(defineProps<Props>(), {
  color: undefined,
  size: '16',
  opacity: 0.8
})
</script>
