<template>
  <div class="settings-switch">
    <div class="info">
      <span class="title text-grey-darken-4">{{ title }}</span>
      <span class="description text-grey-darken-2">{{ description }}</span>
    </div>
    <slot name="default"></slot>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title: string
  description: string
}

defineProps<Props>()
</script>

<style scoped lang="scss">
.settings-switch {
  display: flex;
  flex-direction: row;
  margin: 8px 0;
}

.info {
  flex: 1;
  display: flex;
  flex-direction: column;

  .title {
    font-size: 16px;
  }

  .description {
    font-size: 12px;
  }
}
</style>
