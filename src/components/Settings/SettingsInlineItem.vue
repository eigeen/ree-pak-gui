<template>
  <div class="border-l-2 border-primary/90 pl-4">
    <div v-if="$slots.title || title" class="mb-1 flex items-center gap-2">
      <slot name="title">
        <p class="text-sm font-semibold text-foreground">{{ title }}</p>
      </slot>
    </div>

    <div v-if="$slots.description || description" class="mb-3 text-sm text-muted-foreground">
      <slot name="description">
        <p>{{ description }}</p>
      </slot>
    </div>

    <slot />
  </div>
</template>

<script setup lang="ts">
interface Props {
  title?: string
  description?: string
}

defineProps<Props>()
</script>
