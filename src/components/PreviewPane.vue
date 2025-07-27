<template>
  <v-card class="preview-pane pa-4 elevation-3 rounded-lg">
    <div class="text-subtitle-1">预览</div>
    <v-card-text class="preview-content">
      <!-- 显示预览图片 -->
      <div v-if="previewUri && isImage" class="preview-image-container">
        <el-image
          :src="previewUri"
          :alt="fileName || 'Preview'"
          class="preview-image"
          fit="scale-down"
          :zoom-rate="1.2"
          :max-scale="7"
          :min-scale="0.2"
          :preview-src-list="[previewUri]"
          :initial-index="0"
          show-progress
        >
          <template #error>
            <div class="image-slot">
              <v-icon icon="mdi-alert-circle-outline" size="64" color="error" class="mb-4"></v-icon>
              <p class="text-error text-body-1">预览加载失败</p>
            </div>
          </template>
        </el-image>
        <div v-if="fileName" class="preview-filename">
          {{ fileName }}
        </div>
      </div>
      <!-- 显示其他文件类型的占位符 -->
      <div v-else-if="previewUri && !isImage" class="preview-unsupported">
        <v-icon icon="mdi-file-outline" size="64" color="grey-lighten-1" class="mb-4"></v-icon>
        <p class="text-grey-lighten-1 text-body-1">不支持预览此文件类型</p>
        <p v-if="fileName" class="text-grey-lighten-2 text-body-2">{{ fileName }}</p>
      </div>
      <!-- 默认占位符 -->
      <div v-else class="preview-placeholder">
        <v-icon
          icon="mdi-file-document-outline"
          size="64"
          color="grey-lighten-1"
          class="mb-4"
        ></v-icon>
        <p class="text-grey-lighten-1 text-body-1">选择文件以预览内容</p>
      </div>
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  previewUri?: string
  fileName?: string
}

const props = withDefaults(defineProps<Props>(), {
  previewUri: '',
  fileName: ''
})

// 判断是否为图片文件
const isImage = computed(() => {
  if (!props.previewUri) return false
  const ext = props.previewUri.split('.').pop()?.toLowerCase()
  return ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg'].includes(ext || '')
})
</script>

<style scoped lang="scss">
.preview-pane {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.preview-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.preview-placeholder,
.preview-unsupported {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.preview-image-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  max-width: 100%;
  max-height: 100%;
  width: 100%;
}

.preview-image {
  width: 100%;
  height: calc(100vh - 300px);
  min-height: 400px;
  max-height: 600px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  
  // element-plus 图片组件的自定义样式
  :deep(.el-image__inner) {
    width: 100%;
    height: 100%;
    object-fit: scale-down;
  }
  
  // 错误状态样式
  :deep(.el-image__error) {
    background: transparent;
  }
}

.preview-filename {
  margin-top: 12px;
  font-size: 0.875rem;
  color: rgba(0, 0, 0, 0.6);
  text-align: center;
  word-break: break-all;
  max-width: 100%;
}

.image-slot {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  text-align: center;
}
</style>
