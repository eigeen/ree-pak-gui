<template>
  <div class="tools-view">
    <!-- 工具内容区域 -->
    <div class="tool-container">
      <component v-if="currentTool" :is="currentTool.component" :key="toolId" />
      <div v-else class="tool-not-found">
        <v-alert type="error" variant="tonal">
          <template v-slot:title> 工具不存在 </template>
          找不到ID为 "{{ toolId }}" 的工具
        </v-alert>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getToolById } from '@/config/tools'

const route = useRoute()
const router = useRouter()

// 获取当前工具ID
const toolId = computed(() => route.params.toolId as string)

// 获取当前工具配置
const currentTool = computed(() => {
  return getToolById(toolId.value)
})

// 面包屑导航项
const breadcrumbItems = computed(() => [
  {
    title: '首页',
    to: '/unpack'
  },
  {
    title: '工具',
    disabled: true
  },
  {
    title: currentTool.value?.title || toolId.value,
    disabled: true
  }
])

// 如果工具不存在，重定向到首页
if (!currentTool.value) {
  router.replace('/')
}
</script>

<style scoped lang="scss">
.tools-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.breadcrumb-nav {
  flex: 0 0 auto;
  padding: 12px 16px 8px;
  border-bottom: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}

.tool-container {
  flex: 1;
  overflow: hidden;
  padding: 16px;
}

.tool-not-found {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}
</style>
