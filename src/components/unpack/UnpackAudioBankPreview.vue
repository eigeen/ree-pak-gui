<script setup lang="ts">
import { AudioLines, Music } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import type { ExplorerEntry } from '@/lib/unpackExplorer'

defineProps<{
  entry: ExplorerEntry
}>()

const { t } = useI18n()
</script>

<template>
  <div class="flex h-full flex-col bg-[#0a0a0a] text-foreground">
    <div class="flex min-h-0 flex-1 items-center justify-center p-8">
      <div class="flex max-w-md flex-col items-center gap-3 text-center">
        <div class="flex size-14 items-center justify-center rounded-full bg-[#15282d]">
          <Music class="size-6 text-[#5dccd0]" />
        </div>
        <p class="text-base font-semibold text-foreground">
          {{ t('unpack.audioBankPreviewTitle') }}
        </p>
        <p class="text-sm text-muted-foreground">
          {{ t('unpack.audioBankPreviewPlaceholder') }}
        </p>
        <p class="mt-1 inline-flex items-center gap-1.5 text-xs text-muted-foreground">
          <AudioLines class="size-3.5" />
          {{ entry.name }}
        </p>
      </div>
    </div>
  </div>
</template>
