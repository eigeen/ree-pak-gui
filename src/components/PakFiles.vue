<script setup lang="ts">
import type { PakId, PakInfo } from '@/api/tauri/pak'

export interface Props {
  // 文件列表
  pakList: PakInfo[]
  // 是否允许添加文件
  enableAdd: boolean
}

export interface Data {
  id: PakId
  path: string
}

withDefaults(defineProps<Props>(), {
  enableAdd: true
})

defineEmits(['open', 'close', 'render'])
</script>

<template>
  <el-table class="file-table" :data="pakList" max-height="200" :show-header="false" stripe>
    <el-table-column prop="path" label="Path" />
    <el-table-column label="Operations" fixed="right" min-width="40">
      <template #default="scope">
        <el-button link type="primary" size="small" @click="$emit('close', scope.$index)">
          Close
        </el-button>
      </template>
    </el-table-column>
  </el-table>
  <div class="button-panel">
    <div class="w100">
      <v-btn
        class="button text-none"
        color="#409eff"
        prepend-icon="mdi-file-plus-outline"
        @click="$emit('open')"
        :disabled="!enableAdd"
      >
        Open Paks
      </v-btn>
      <v-tooltip activator="parent" location="top" :disabled="enableAdd"
        >Select a File Name Table first.</v-tooltip
      >
    </div>
  </div>
</template>

<style scoped lang="scss">
.file-table {
  width: 100%;
  border: 1px solid var(--el-border-color);
  border-radius: var(--el-border-radius-base);
  height: 200px;
}

.button-panel {
  display: flex;
  flex-flow: row;
  justify-content: space-between;

  .button {
    flex: 1;
    width: 100%;
  }
  .button-right {
    margin-left: 10px;
  }
}

.w100 {
  width: 100%;
}
</style>
