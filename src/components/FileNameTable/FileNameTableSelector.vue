<template>
  <div :class="props.showLabel ? 'space-y-2' : ''">
    <label v-if="props.showLabel" class="text-sm font-medium text-foreground">Path List</label>

    <Popover v-model:open="open">
      <PopoverTrigger as-child>
        <Button
          variant="outline"
          role="combobox"
          :aria-expanded="open"
          class="h-8 w-full justify-between rounded-md border-border/80 bg-background px-3 text-sm font-normal text-foreground shadow-none hover:bg-secondary/70"
        >
          <span class="truncate text-left">
            {{ selectedLabel || props.placeholder }}
          </span>
          <ChevronsUpDown class="ml-2 size-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>

      <PopoverContent
        align="start"
        class="w-[var(--reka-popper-anchor-width)] rounded-md border border-border/80 bg-popover p-0 shadow-md"
      >
        <div class="border-b border-border/80 p-2">
          <div class="relative">
            <Search class="pointer-events-none absolute left-2.5 top-2 size-4 text-[#8b949e]" />
            <DenseInput v-model="searchText" class="pl-8" placeholder="搜索 Path List..." />
          </div>
        </div>

        <div class="editor-scrollbar max-h-64 overflow-auto p-1">
          <div
            v-if="filteredItems.length === 0"
            class="px-3 py-6 text-center text-sm text-muted-foreground"
          >
            没有匹配的文件名表
          </div>

          <button
            v-for="item in filteredItems"
            :key="item.value"
            type="button"
            class="flex w-full items-center gap-2 rounded-sm px-2.5 py-2 text-left text-sm text-foreground transition-colors hover:bg-secondary/80"
            :class="selectedValue === item.value ? 'bg-secondary text-foreground' : ''"
            @click="selectItem(item.value)"
          >
            <Check
              :class="
                cn(
                  'size-4 shrink-0 text-primary transition-opacity',
                  selectedValue === item.value ? 'opacity-100' : 'opacity-0'
                )
              "
            />
            <span class="truncate">{{ item.label }}</span>
          </button>
        </div>
      </PopoverContent>
    </Popover>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Check, ChevronsUpDown, Search } from 'lucide-vue-next'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { DenseInput } from '@/components/ui/input'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'

export interface Option {
  label: string
  value: string
}

const selectedValue = defineModel<string>({ default: '' })
const props = withDefaults(
  defineProps<{
    items: Option[]
    showLabel?: boolean
    placeholder?: string
  }>(),
  {
    showLabel: true,
    placeholder: '请选择文件名表'
  }
)

const open = ref(false)
const searchText = ref('')

const selectedLabel = computed(
  () => props.items.find((item) => item.value === selectedValue.value)?.label ?? ''
)

const filteredItems = computed(() => {
  const keyword = searchText.value.trim().toLowerCase()
  if (!keyword) {
    return props.items
  }

  return props.items.filter((item) => item.label.toLowerCase().includes(keyword))
})

watch(open, (nextOpen) => {
  if (!nextOpen) {
    searchText.value = ''
  }
})

function selectItem(value: string) {
  selectedValue.value = selectedValue.value === value ? '' : value
  open.value = false
}
</script>
