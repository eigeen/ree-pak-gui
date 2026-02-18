<template>
  <v-card class="elevation-0 file-table">
    <draggable
      v-model="orderedPakList"
      item-key="path"
      :animation="200"
      handle=".drag-handle"
      ghost-class="ghost"
      :forceFallback="true"
      @change="onChange"
    >
      <template #item="{ element, index }">
        <div>
          <PakFileItem :file-path="element.path" @remove="$emit('close', index)"></PakFileItem>
        </div>
      </template>
    </draggable>
  </v-card>
  <div class="button-panel">
    <div style="flex: 1 1 auto">
      <v-btn
        class="button long-btn text-none"
        style="width: 100%"
        color="primary"
        prepend-icon="mdi-file-plus-outline"
        @click="$emit('open')"
        :disabled="!enableAdd"
      >
        {{ t('pakFiles.openPaks') }}
      </v-btn>
      <v-tooltip activator="parent" location="top" :disabled="enableAdd">
        {{ t('pakFiles.selectFileNameTable') }}
      </v-tooltip>
    </div>
    <div>
      <v-btn
        class="button mg-l-10 short-btn text-none"
        icon="mdi-close-box-multiple"
        rounded="rounded"
        size="small"
        :disabled="pakList.length === 0"
        @click="$emit('closeAll')"
      >
      </v-btn>
      <v-tooltip activator="parent" location="top" :disabled="pakList.length === 0">
        {{ t('pakFiles.closeAllPaks') }}
      </v-tooltip>
    </div>
  </div>
</template>

<script setup lang="ts">
import { pak_order, type PakId, type PakInfo } from '@/api/tauri/pak'
import PakFileItem from '@/components/PakFileItem.vue'
import { ref, watch } from 'vue'
import draggable from 'vuedraggable'
import { useI18n } from 'vue-i18n'
const { t } = useI18n()

export interface Props {
  // 文件列表
  pakList: PakInfo[]
  // 是否允许添加文件
  enableAdd: boolean
}

export interface OrderedData {
  id: PakId
  path: string
}

const props = withDefaults(defineProps<Props>(), {
  enableAdd: true
})

const emit = defineEmits(['open', 'close', 'order', 'closeAll'])

const orderedPakList = ref<OrderedData[]>([])

watch(
  () => props.pakList,
  (newValue: PakInfo[]) => {
    orderedPakList.value = newValue.map((pak, index) => ({
      id: pak.id,
      path: pak.path
    }))
  }
)

// 监听拖拽变化事件
async function onChange(event: any) {
  console.log('Draggable change event:', event)
  if (!event.moved) {
    return
  }
  const { oldIndex, newIndex } = event
  if (typeof oldIndex !== 'number' || typeof newIndex !== 'number') {
    return
  }
  // create ordered list
  const newList = [...orderedPakList.value]
  const item = newList.splice(oldIndex, 1)[0]
  if (!item) {
    return
  }
  newList.splice(newIndex, 0, item)
  const orderList = newList.map((item) => item.id)
  // send order list to backend
  await pak_order(orderList)
  emit('order', orderList)
}
</script>

<style scoped lang="scss">
.file-table {
  width: 100%;
  border: 1px solid var(--el-border-color);
  border-radius: var(--el-border-radius-base);
  height: 200px;
  overflow-y: auto;
  padding: 8px;
}

.ghost {
  opacity: 0.5;
  background: #eee;
}

.button-panel {
  display: flex;
  flex-flow: row;
  justify-content: space-between;
  width: 100%;

  .button {
    min-width: auto;
  }
  .long-btn {
    flex: 1 1 auto;
    width: 100%;
  }
  .mg-l-10 {
    margin-left: 10px;
  }
  .short-btn {
    width: 36px;
    height: 36px;
    flex: 0 0 auto;
  }
}

.w100 {
  width: 100%;
}
</style>
