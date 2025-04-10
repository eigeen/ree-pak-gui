<template>
  <!-- 文件名 操作 -->
  <div class="file-item">
    <v-icon icon="mdi-drag" class="drag-handle"></v-icon>
    <div class="file-info">
      <span class="file-name">
        {{ fileName }}
        <v-tooltip activator="parent" location="right" :disabled="fileName === filePath">{{
          filePath
        }}</v-tooltip>
      </span>
    </div>
    <v-btn
      class="remove-btn"
      icon="mdi-close"
      variant="plain"
      size="small"
      density="compact"
      @click="$emit('remove')"
    ></v-btn>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

export interface Props {
  filePath: string
}

const props = defineProps<Props>()

defineEmits(['remove'])

const fileName = computed(() => {
  const parts = props.filePath.split(/[\\/]/) // split by slash
  if (parts.length === 1) {
    return props.filePath
  }
  return parts[parts.length - 1]
})
</script>

<style scoped>
.file-item {
  display: flex;
  align-items: center;
  width: 100%;
  height: 30px;
}

.drag-handle {
  cursor: move;
  margin-right: 6px;
}

.file-info {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.remove-btn {
  flex-shrink: 0;
}

.file-name {
  font-size: 0.9rem;
}
</style>
