<template>
  <v-autocomplete
    label="Path List"
    v-model="selectedValue"
    :items="items"
    item-title="label"
    item-value="value"
    density="comfortable"
    variant="outlined"
    persistent-hint
    :hide-details="hintData === null"
  >
  </v-autocomplete>
</template>

<script setup lang="ts">
import { ref } from 'vue'

export interface Option {
  label: string
  value: string
}

const selectedValue = defineModel<string>()
const props = defineProps<{ items: Option[] }>()

const hintData = ref<string | null>(null)
</script>

<style scoped></style>
