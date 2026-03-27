<template>
  <TooltipProvider>
    <Tooltip>
      <TooltipTrigger as-child>
        <button
          class="inline-flex items-center justify-center rounded-full text-muted-foreground transition hover:text-foreground"
          :style="{ opacity }"
          type="button"
        >
          <CircleHelp :size="size" :color="color" />
        </button>
      </TooltipTrigger>
      <TooltipContent class="max-w-72 rounded-xl px-3 py-2 text-sm leading-6">
        <slot />
      </TooltipContent>
    </Tooltip>
  </TooltipProvider>
</template>

<script setup lang="ts">
import { CircleHelp } from 'lucide-vue-next'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

interface Props {
  color?: string
  size?: number
  opacity?: number
}

withDefaults(defineProps<Props>(), {
  color: undefined,
  size: 16,
  opacity: 0.8
})
</script>
