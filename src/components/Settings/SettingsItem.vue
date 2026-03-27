<template>
  <div
    class="flex items-start justify-between gap-4 rounded-2xl border border-border/70 bg-secondary/25 p-4"
  >
    <div class="space-y-1">
      <h3 class="text-sm font-semibold tracking-tight text-foreground">{{ title }}</h3>
      <p class="text-sm leading-6 text-muted-foreground">{{ description }}</p>
    </div>
    <div class="shrink-0">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title: string
  description: string
}

defineProps<Props>()
</script>
