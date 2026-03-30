<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog'

export type PropertyRow = {
  key: string
  label: string
  value: string
}

export type PropertySection = {
  key: string
  title?: string
  rows: PropertyRow[]
}

const props = defineProps<{
  open: boolean
  title: string
  description?: string
  loading?: boolean
  sections: PropertySection[]
  emptyText?: string
}>()

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void
}>()
</script>

<template>
  <Dialog :open="props.open" @update:open="emit('update:open', $event)">
    <DialogContent
      class="flex max-h-[calc(100vh-2rem)] w-[min(56rem,calc(100vw-2rem))] max-w-none flex-col gap-0 overflow-hidden rounded-[1rem] border-border/80 bg-background/96 p-0"
    >
      <DialogHeader class="min-w-0 border-b border-border/80 px-6 py-5 pr-14">
        <DialogTitle class="min-w-0 pr-2 leading-6">{{ props.title }}</DialogTitle>
        <DialogDescription v-if="props.description" class="min-w-0 break-all">
          {{ props.description }}
        </DialogDescription>
      </DialogHeader>

      <div class="editor-scrollbar min-h-0 min-w-0 flex-1 overflow-y-auto overflow-x-hidden px-6 py-5">
        <div v-if="props.loading" class="empty-state min-h-40 border-border/70">
          <p class="text-sm font-medium text-foreground">正在读取属性…</p>
          <p class="section-copy">这可能需要一点时间。</p>
        </div>

        <div
          v-else-if="props.sections.length === 0"
          class="empty-state min-h-40 border-border/70"
        >
          <p class="text-sm font-medium text-foreground">
            {{ props.emptyText ?? '没有可显示的属性。' }}
          </p>
        </div>

        <div v-else class="space-y-4">
          <section
            v-for="section in props.sections"
            :key="section.key"
            class="min-w-0 overflow-hidden rounded-[0.85rem] border border-border/80 bg-card/72"
          >
            <div
              v-if="section.title"
              class="border-b border-border/70 px-4 py-3 text-sm font-semibold text-foreground"
            >
              {{ section.title }}
            </div>

            <div class="min-w-0">
              <template v-for="row in section.rows" :key="row.key">
                <div
                  class="grid min-w-0 gap-2 border-t border-border/60 px-4 py-3 first:border-t-0 sm:grid-cols-[minmax(120px,0.32fr)_minmax(0,1fr)] sm:gap-4"
                >
                  <div
                    class="min-w-0 text-xs font-medium tracking-[0.08em] text-muted-foreground uppercase"
                  >
                    {{ row.label }}
                  </div>
                  <div class="min-w-0 break-all text-sm text-foreground">
                    {{ row.value }}
                  </div>
                </div>
              </template>
            </div>
          </section>
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>
