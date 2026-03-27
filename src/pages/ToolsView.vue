<template>
  <section class="space-y-4">
    <div class="space-y-1">
      <p class="section-eyebrow">Tools</p>
      <h2 class="section-title">{{ currentTool ? t(currentTool.title) : 'Unknown Tool' }}</h2>
      <p class="section-copy">独立工具页保留业务能力，只替换视觉壳与交互原语。</p>
    </div>

    <div class="app-panel min-h-[calc(100vh-14rem)] p-4 sm:p-6">
      <component v-if="currentTool" :is="currentTool.component" :key="toolId" />
      <div v-else class="empty-state">
        <p class="text-base font-semibold text-foreground">工具不存在</p>
        <p class="section-copy">找不到 ID 为 "{{ toolId }}" 的工具。</p>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { getToolById } from '@/config/tools'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()

const toolId = computed(() => route.params.toolId as string)
const currentTool = computed(() => getToolById(toolId.value))

if (!currentTool.value) {
  router.replace('/')
}
</script>
