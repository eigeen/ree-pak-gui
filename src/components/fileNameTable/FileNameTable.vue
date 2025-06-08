<template>
  <div class="file-name-table">
    <FileNameTableSelector v-model="selectedValue"></FileNameTableSelector>
  </div>
</template>

<script setup lang="ts">
const selectedValue = defineModel<string>()
</script>

<style scoped>
.file-name-table {
  display: flex;
  flex-direction: row;
}
</style>
