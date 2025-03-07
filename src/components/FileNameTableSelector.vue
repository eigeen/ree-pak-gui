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
const selectedValue = ref('')
const options = ref<Option[]>([])

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
  <el-select
    placeholder="Select"
    v-model="selectedValue"
    @change="$emit('change', selectedValue)"
    :disabled="disabled"
  >
    <el-option v-for="item in options" :key="item.value" :label="item.label" :value="item.value" />
  </el-select>
</template>

<style scoped></style>
