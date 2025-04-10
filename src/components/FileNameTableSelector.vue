<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { file_table_get_list } from '@/api/tauri/filelist'

interface Option {
  label: string
  value: string
}

interface Props {
  disabled: boolean
}

defineEmits(['change'])

withDefaults(defineProps<Props>(), {
  disabled: false
})
const selectedValue = ref<string | null>(null)
const options = ref<Option[]>([])
const hintData = ref<string | null>(null)

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
  <!-- <el-select
    placeholder="Select"
    v-model="selectedValue"
    @change="$emit('change', selectedValue)"
    :disabled="disabled"
  >
    <el-option v-for="item in options" :key="item.value" :label="item.label" :value="item.value" />
  </el-select> -->
  <v-autocomplete
    label="File Name Table"
    v-model="selectedValue"
    @change="$emit('change', selectedValue)"
    :disabled="disabled"
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
