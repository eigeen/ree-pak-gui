<template>
  <Sheet v-model:open="showSettings">
    <SheetContent
      side="right"
      class="w-full gap-0 border-l-white/40 bg-background/95 px-0 sm:max-w-md"
    >
      <SheetHeader class="border-b border-border/70 px-6 pb-5">
        <SheetTitle>Settings</SheetTitle>
        <SheetDescription>应用行为与本地持久化设置。</SheetDescription>
      </SheetHeader>

      <div class="flex flex-1 flex-col gap-4 px-6 py-6">
        <SettingsItemSwitch
          v-model="autoSave"
          title="自动保存"
          description="工作区与设置发生变化后自动写入本地配置文件。"
        />

        <Separator />

        <div class="app-panel-muted p-4">
          <p class="text-sm font-medium text-foreground">设置版本</p>
          <p class="mt-1 text-sm text-muted-foreground">
            {{ settingsValue?.version ?? '1' }}
          </p>
        </div>
      </div>
    </SheetContent>
  </Sheet>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useSettingsStore } from '@/store/settings'
import { Separator } from '@/components/ui/separator'
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle
} from '@/components/ui/sheet'

const settingsStore = useSettingsStore()
const showSettings = computed({
  get: () => settingsStore.showSettings as unknown as boolean,
  set: (value: boolean) => {
    ;(settingsStore as any).showSettings = value
  }
})
const autoSave = computed({
  get: () => settingsStore.autoSave as unknown as boolean,
  set: (value: boolean) => {
    ;(settingsStore as any).autoSave = value
  }
})
const settingsValue = computed(() => settingsStore.settings as unknown as { version: string })
</script>
