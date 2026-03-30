<template>
  <div
    class="surface-interactive group select-none flex items-center gap-3 rounded-[0.95rem] px-3 py-3 transition-colors"
    @contextmenu="$emit('contextmenu', $event)"
  >
    <div
      class="drag-handle flex size-9 shrink-0 items-center justify-center rounded-2xl text-muted-foreground transition-colors group-hover:border-primary/20 group-hover:text-foreground"
    >
      <GripVertical class="size-4" />
    </div>
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger as-child>
          <div class="min-w-0 flex-1 cursor-default select-none">
            <div class="flex items-center gap-2">
              <p class="truncate text-sm font-medium text-foreground">{{ fileName }}</p>
            </div>
            <p class="truncate text-xs text-muted-foreground">{{ filePath }}</p>
          </div>
        </TooltipTrigger>
        <TooltipContent
          v-if="fileName !== filePath"
          class="max-w-[28rem] rounded-xl px-3 py-2 text-sm"
        >
          {{ fileName }}
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
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

export interface Props {
  pakId: string
  filePath: string
}

const props = defineProps<Props>()

defineEmits<{
  (e: 'remove'): void
  (e: 'contextmenu', event: MouseEvent): void
}>()

const fileName = computed(() => {
  const parts = props.filePath.split(/[\\/]/)
  return parts.length === 1 ? props.filePath : (parts[parts.length - 1] ?? props.filePath)
})
</script>
