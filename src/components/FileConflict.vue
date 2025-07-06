<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { ConflictFile } from '@/lib/packer'

const conflicts = defineModel<ConflictFile[]>('conflicts')

// 本地状态
const expandedFiles = ref<Set<string>>(new Set())
const localConflicts = ref<ConflictFile[]>([])

// 监听props变化，同步到本地状态
watch(
  conflicts,
  (newConflicts) => {
    if (!newConflicts) return

    localConflicts.value = newConflicts.map((conflict) => ({
      ...conflict,
      selectedSource: conflict.selectedSource ?? conflict.sources.length - 1 // 默认选择最后一个来源
    }))
  },
  { immediate: true }
)

// 方法
const toggleExpanded = (relativePath: string) => {
  if (expandedFiles.value.has(relativePath)) {
    expandedFiles.value.delete(relativePath)
  } else {
    expandedFiles.value.add(relativePath)
  }
}

const isExpanded = (relativePath: string) => {
  return expandedFiles.value.has(relativePath)
}

const formatFileSize = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  const value = bytes / Math.pow(k, i)
  const formattedValue = i === 0 ? Math.round(value) : parseFloat(value.toFixed(2))

  return formattedValue + ' ' + sizes[i]
}

const formatDate = (date: Date) => {
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}
</script>

<template>
  <div>
    <div class="text-body-1 mb-4">检测到以下文件存在冲突，请选择要保留的文件版本：</div>

    <div class="max-h-[500px] overflow-y-auto">
      <div v-for="conflict in localConflicts" :key="conflict.relativePath" class="mb-2">
        <!-- 文件路径行 -->
        <div class="flex items-start align-center gap-2 bg-grey-lighten-5 rounded">
          <v-btn
            :icon="isExpanded(conflict.relativePath) ? 'mdi-chevron-down' : 'mdi-chevron-right'"
            variant="text"
            size="small"
            @click="toggleExpanded(conflict.relativePath)"
          ></v-btn>

          <v-icon icon="mdi-file-alert" color="warning" size="small"></v-icon>

          <div class="flex-1 min-w-0">
            <span class="text-body-1 font-medium break-all">{{ conflict.relativePath }}</span>
            <div class="flex gap-4 text-caption text-grey-darken-1 mt-1">
              <span v-if="conflict.size">大小: {{ formatFileSize(conflict.size) }}</span>
              <span v-if="conflict.modifiedDate"
                >修改时间: {{ formatDate(conflict.modifiedDate) }}</span
              >
            </div>
          </div>

          <v-chip size="small" color="warning" variant="outlined" class="flex-shrink-0">
            {{ conflict.sources.length }} 个来源
          </v-chip>
        </div>

        <!-- 展开的文件来源详情 -->
        <v-expand-transition>
          <div v-if="isExpanded(conflict.relativePath)" class="ml-4 border-l-1 pl-4">
            <v-radio-group
              v-model="conflict.selectedSource"
              density="compact"
              class="mt-2"
              hide-details
            >
              <!-- 固定选项：移除该文件 -->
              <div class="mb-2">
                <v-radio :value="-1" class="mb-1" hide-details>
                  <template #label>
                    <div class="flex items-center gap-2">
                      <v-icon icon="mdi-delete" color="error" size="small"></v-icon>
                      <span class="text-body-2 font-medium text-error">移除该文件</span>
                    </div>
                  </template>
                </v-radio>
              </div>

              <!-- 文件来源选项 -->
              <div v-for="(source, index) in conflict.sources" :key="index" class="mb-2">
                <v-radio :value="index" class="mb-1" hide-details>
                  <template #label>
                    <div class="flex-1 min-w-0">
                      <div class="text-body-2 font-medium break-all">{{ source.sourcePath }}</div>
                    </div>
                  </template>
                </v-radio>
              </div>
            </v-radio-group>
          </div>
        </v-expand-transition>
      </div>
    </div>
  </div>
</template>
