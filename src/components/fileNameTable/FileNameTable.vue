<template>
  <FileNameTableSelector @change="onFileNameTableChange"></FileNameTableSelector>
</template>

<script setup lang="ts">
const emit = defineEmits(['change'])

function onFileNameTableChange(filePath: string) {
  emit('change', filePath)
}
</script>

<style scoped></style>
