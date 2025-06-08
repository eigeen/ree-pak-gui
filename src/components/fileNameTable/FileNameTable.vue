<template>
  <FileNameTableSelector v-model="selectedValue"></FileNameTableSelector>
</template>

<script setup lang="ts">
const selectedValue = defineModel<string>()
</script>

<style scoped></style>
