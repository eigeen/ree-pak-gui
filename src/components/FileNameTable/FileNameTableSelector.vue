<template>
  <div class="space-y-2">
    <label class="text-sm font-medium text-foreground">Path List</label>
    <div class="relative">
      <select
        v-model="selectedValue"
        class="flex h-10 w-full rounded-xl border border-input bg-background px-3 py-2 text-sm shadow-xs outline-none transition focus:border-ring focus:ring-2 focus:ring-ring/30"
      >
        <option value="">请选择文件名表</option>
        <option v-for="item in items" :key="item.value" :value="item.value">
          {{ item.label }}
        </option>
      </select>
    </div>
  </div>
</template>

<script setup lang="ts">
export interface Option {
  label: string
  value: string
}

const selectedValue = defineModel<string>({ default: '' })

defineProps<{ items: Option[] }>()
</script>
