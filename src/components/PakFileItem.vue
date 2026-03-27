<template>
  <div
    class="flex items-center gap-3 rounded-2xl border border-border/70 bg-background/85 px-3 py-2.5"
  >
    <GripVertical class="drag-handle size-4 shrink-0 text-muted-foreground" />
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger as-child>
          <div class="min-w-0 flex-1 cursor-default">
            <p class="truncate text-sm font-medium text-foreground">{{ fileName }}</p>
            <p class="truncate text-xs text-muted-foreground">{{ filePath }}</p>
          </div>
        </TooltipTrigger>
        <TooltipContent
          v-if="fileName !== filePath"
          class="max-w-[28rem] rounded-xl px-3 py-2 text-sm"
        >
          {{ filePath }}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
    <Button size="icon-sm" variant="ghost" class="rounded-full" @click="$emit('remove')">
      <X class="size-4" />
    </Button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { GripVertical, X } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

export interface Props {
  filePath: string
}

const props = defineProps<Props>()

defineEmits(['remove'])

const fileName = computed(() => {
  const parts = props.filePath.split(/[\\/]/)
  return parts.length === 1 ? props.filePath : (parts[parts.length - 1] ?? props.filePath)
})
</script>
