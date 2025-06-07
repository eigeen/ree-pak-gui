<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { file_table_get_list } from '@/api/tauri/filelist'

interface Option {
  label: string
  value: string
}

const emit = defineEmits(['change'])

const selectedValue = ref<string | null>(null)
const options = ref<Option[]>([])
const hintData = ref<string | null>(null)

watch(selectedValue, (newValue) => {
  if (newValue !== null) {
    emit('change', newValue)
  }
})

onMounted(async () => {
  try {
    const file_table_list = await file_table_get_list()
    options.value = file_table_list.map((file_table) => ({
      label: file_table.name,
      value: file_table.absPath
    }))
  } catch (error) {
    console.error(error)
  }
})
</script>

<template>
  <v-autocomplete
    label="File Name Table"
    v-model="selectedValue"
    :items="options"
    item-title="label"
    item-value="value"
    density="comfortable"
    variant="outlined"
    persistent-hint
    :hide-details="hintData === null"
  >
  </v-autocomplete>
</template>

<style scoped></style>
