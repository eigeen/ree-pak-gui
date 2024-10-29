<script setup lang="ts">
import type { PakId, PakInfo } from '@/api/tauri/pak';

export interface Props {
  // 文件列表
  pakList: PakInfo[];
  // 是否允许添加文件
  enableAdd: boolean;
}

export interface Data {
  id: PakId;
  path: string;
}

withDefaults(defineProps<Props>(), {
  enableAdd: true,
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
    <el-tooltip class="box-item" effect="dark" content="Please select a File Name Table before adding paks"
      placement="top-start" :disabled="enableAdd">
      <el-button class="button" type="primary" @click="$emit('open')" :disabled="!enableAdd">Add Files</el-button>
    </el-tooltip>
    <el-button class="button" type="success" @click="$emit('render')" :disabled="!enableAdd">Render Tree</el-button>
  </div>
</template>

<style scoped>
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
}

.button-panel .button {
  flex: 1;
}
</style>