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

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { file_table_get_list } from '@/api/tauri/filelist'

interface Option {
  label: string
  value: string
}

const selectedValue = defineModel<string>()

const options = ref<Option[]>([])
const hintData = ref<string | null>(null)

onMounted(async () => {
  const fileTableList = await file_table_get_list()
  options.value = fileTableList.map((fileTable) => ({
    label: fileTable.name,
    value: fileTable.absPath
  }))
})
</script>

<style scoped></style>
