<template>
  <section class="space-y-4">
    <div class="space-y-1">
      <p class="section-eyebrow">{{ t('toolPage.eyebrow') }}</p>
      <h2 class="section-title">
        {{ currentTool ? t(currentTool.title) : t('global.unknownTool') }}
      </h2>
      <p class="section-copy">{{ t('toolPage.intro') }}</p>
    </div>

    <div class="app-panel min-h-[calc(100vh-14rem)] p-4 sm:p-6">
      <component v-if="currentTool" :is="currentTool.component" :key="toolId" />
      <div v-else class="empty-state">
        <p class="text-base font-semibold text-foreground">{{ t('toolPage.missingTitle') }}</p>
        <p class="section-copy">{{ t('toolPage.missingDescription', { id: toolId }) }}</p>
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
