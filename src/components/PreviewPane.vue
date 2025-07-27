<template>
  <v-card class="preview-pane pa-4 elevation-3 rounded-lg">
    <div class="text-subtitle-1">预览</div>
    <v-card-text class="preview-content">
      <div class="preview-placeholder">
        <v-icon icon="mdi-file-document-outline" size="64" color="grey-lighten-1" class="mb-4"></v-icon>
        <p class="text-grey-lighten-1 text-body-1">选择文件以预览内容</p>
      </div>
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
// 预留接口，后续可以添加文件选择和预览逻辑
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

.preview-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}
</style> 