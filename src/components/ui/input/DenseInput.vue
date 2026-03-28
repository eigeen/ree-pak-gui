<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { useVModel } from '@vueuse/core'
import { cn } from '@/lib/utils'

const props = defineProps<{
  defaultValue?: string | number
  modelValue?: string | number
  class?: HTMLAttributes['class']
}>()

const emits = defineEmits<{
  (e: 'update:modelValue', payload: string | number): void
}>()

const modelValue = useVModel(props, 'modelValue', emits, {
  passive: true,
  defaultValue: props.defaultValue
})
</script>

<template>
  <input
    v-model="modelValue"
    data-slot="dense-input"
    :class="
      cn(
        'file:text-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input h-7 w-full min-w-0 rounded-[3px] border bg-transparent px-2 py-0 text-sm text-foreground shadow-none transition-[border-color,background-color] outline-none file:inline-flex file:h-5 file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-[#8b949e] disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50',
        'focus-visible:border-primary/70 focus-visible:ring-0',
        'aria-invalid:border-destructive',
        props.class
      )
    "
  />
</template>
