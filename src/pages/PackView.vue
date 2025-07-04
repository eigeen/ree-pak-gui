<script setup lang="ts">
import { ShowError, ShowWarn } from '@/utils/message'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { exists, stat } from '@tauri-apps/plugin-fs'
import { computed, onMounted, onUnmounted, ref } from 'vue'

// 文件列表
interface FileItem {
  path: string
  isDirectory: boolean
}

const inputFiles = ref<FileItem[]>([])
const exportMode = ref<'individual' | 'single'>('individual')
const autoDetectRoot = ref(true)
const exportDirectory = ref('')

// 导出进度
const exportWorking = ref(false)
const showProgress = ref(false)
const currentFile = ref('')
const totalFileCount = ref(0)
const finishFileCount = ref(0)
const progressValue = computed(() =>
  totalFileCount.value === 0 ? 0 : (finishFileCount.value / totalFileCount.value) * 100
)

// 导出结果
const exportResult = ref<{
  success: boolean
  files: string[]
  error: string
}>({
  success: false,
  files: [],
  error: ''
})

// 计算属性
const enableExport = computed(() => {
  return inputFiles.value.length > 0 && exportDirectory.value !== ''
})

// Add files or folders
const addFiles = async (paths: string[]) => {
  try {
    for (const path of paths) {
      if (!(await exists(path))) {
        ShowError(`输入文件 ${path} 不存在。`)
        continue
      }

      const st = await stat(path)
      // warning if the file is a pak file
      if (st.isFile && path.endsWith('.pak')) {
        ShowWarn(`输入文件 ${path} 可能是 pak 文件。当前为打包页面，如需解包请使用解包功能。`)
      }

      inputFiles.value.push({
        path,
        isDirectory: st.isDirectory
      })
    }
  } catch (e) {
    ShowError(e)
  }
}

// 处理添加文件
const handleAddViaDialog = async (folder: boolean) => {
  const results = await openDialog({
    multiple: true,
    directory: folder
  })
  if (!results) return

  await addFiles(results)
}

// 处理选择导出目录
const handleSelectDirectory = () => {
  console.log('选择导出目录')
  // TODO: 实现目录选择对话框
}

// 处理移除文件
const handleRemoveFile = (index: number) => {
  console.log('移除文件', index)
  inputFiles.value.splice(index, 1)
}

// 处理导出
const handleExport = () => {
  console.log('开始导出', {
    files: inputFiles.value,
    mode: exportMode.value,
    autoDetectRoot: autoDetectRoot.value,
    exportDirectory: exportDirectory.value
  })
  // TODO: 实现导出功能

  // 重置导出状态
  exportResult.value = {
    success: false,
    files: [],
    error: ''
  }

  // 模拟导出进度
  exportWorking.value = true
  showProgress.value = true
  totalFileCount.value = inputFiles.value.length
  finishFileCount.value = 0

  // 模拟进度更新
  const interval = setInterval(() => {
    finishFileCount.value++
    currentFile.value = `file_${finishFileCount.value}.pak`

    if (finishFileCount.value >= totalFileCount.value) {
      clearInterval(interval)
      exportWorking.value = false
      exportResult.value = {
        success: true,
        files: ['output1.pak', 'output2.pak'],
        error: ''
      }
    }
  }, 500)
}

// 处理重置导出状态
const handleResetExport = () => {
  exportWorking.value = false
  showProgress.value = false
  exportResult.value = {
    success: false,
    files: [],
    error: ''
  }
  totalFileCount.value = 0
  finishFileCount.value = 0
  currentFile.value = ''
}

const dropInAddFiles = async (paths: string[]) => {
  console.debug('drop in add files', paths)
}

let unlisten: UnlistenFn | undefined
const startListenToDrop = async () => {
  unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type === 'drop') {
      await dropInAddFiles(event.payload.paths)
    }
  })
}

const stopListenToDrop = () => {
  unlisten?.()
}

onMounted(async () => {
  await startListenToDrop()
})

onUnmounted(() => {
  stopListenToDrop()
})
</script>

<template>
  <div class="h-full flex">
    <!-- 左侧文件列表 -->
    <div class="flex-1 h-full pr-2 pb-4">
      <v-card class="pa-4 elevation-3 rounded-lg h-full">
        <!-- 添加文件按钮 -->
        <div class="flex gap-2 mb-4">
          <v-btn
            class="text-none"
            color="primary"
            prepend-icon="mdi-file-plus"
            @click="handleAddViaDialog(false)"
          >
            添加文件
          </v-btn>
          <v-btn
            class="text-none"
            color="primary"
            prepend-icon="mdi-folder-plus"
            @click="handleAddViaDialog(true)"
          >
            添加文件夹
          </v-btn>
        </div>

        <!-- 文件列表 -->
        <div class="text-subtitle-1 mb-4">文件列表</div>
        <div class="h-[calc(100vh-280px)] overflow-auto">
          <div
            v-if="inputFiles.length === 0"
            class="flex flex-col items-center justify-center h-full text-center"
          >
            <v-icon icon="mdi-file-outline" size="64" color="grey-lighten-1" class="mb-4"></v-icon>
            <p class="text-grey-lighten-1 text-h6">尚未添加文件或文件夹</p>
            <p class="text-grey-lighten-1 text-body-2">点击上方按钮或拖拽文件到此处添加</p>
          </div>

          <div v-else class="h-full overflow-auto">
            <v-list>
              <v-list-item
                v-for="(file, index) in inputFiles"
                :key="index"
                class="border-b border-gray-100"
              >
                <template #prepend>
                  <v-icon
                    :icon="file.isDirectory ? 'mdi-folder' : 'mdi-file'"
                    :color="file.isDirectory ? 'blue' : 'grey'"
                  ></v-icon>
                </template>

                <v-list-item-title class="font-mono text-sm break-all">
                  {{ file.path }}
                </v-list-item-title>

                <template #append>
                  <v-btn
                    icon="mdi-close"
                    variant="text"
                    size="small"
                    @click="handleRemoveFile(index)"
                  ></v-btn>
                </template>
              </v-list-item>
            </v-list>
          </div>
        </div>
      </v-card>
    </div>

    <!-- 右侧导出设置 -->
    <div class="w-[350px] flex flex-col gap-4 pl-2">
      <!-- 导出配置 -->
      <v-card class="pa-4 elevation-3 rounded-lg">
        <div class="text-subtitle-1 mb-4">导出设置</div>

        <!-- 导出模式 -->
        <div class="mb-4">
          <div class="text-body-2 mb-2">导出模式</div>
          <v-radio-group v-model="exportMode" density="compact" hide-details>
            <v-radio label="每个文件项单独导出 pak" value="individual" density="compact"></v-radio>
            <v-radio label="所有文件导出为单个 pak" value="single" density="compact"></v-radio>
          </v-radio-group>
        </div>

        <!-- 导出配置 -->
        <div class="mb-4">
          <v-checkbox
            v-model="autoDetectRoot"
            label="自动检测根目录"
            density="compact"
            color="primary"
            hide-details
          ></v-checkbox>
        </div>

        <!-- 导出文件 -->
        <div class="mb-4">
          <div class="text-body-2 mb-2">导出文件</div>
          <div class="flex gap-2">
            <v-text-field
              v-model="exportDirectory"
              variant="outlined"
              density="comfortable"
              hide-details
              placeholder="导出目录"
            ></v-text-field>
            <v-btn icon="mdi-folder-open" variant="outlined" @click="handleSelectDirectory"></v-btn>
          </div>
        </div>

        <!-- 导出按钮 -->
        <v-btn
          class="text-none"
          color="primary"
          prepend-icon="mdi-export"
          @click="handleExport"
          :disabled="!enableExport"
          block
        >
          导出
        </v-btn>

        <!-- 导出进度条 -->
        <div
          v-if="exportWorking || showProgress || exportResult.success || exportResult.error"
          class="mt-4"
        >
          <v-progress-linear
            v-if="exportWorking || showProgress"
            :color="progressValue >= 100 ? 'green' : 'primary'"
            height="12px"
            :model-value="progressValue"
            rounded
            class="mb-2"
          ></v-progress-linear>

          <!-- 进度信息 -->
          <div v-if="exportWorking || showProgress" class="text-body-2 mb-2">
            {{ finishFileCount }} / {{ totalFileCount }} 个文件
          </div>
          <div v-if="exportWorking || showProgress" class="text-body-2 mb-1">正在导出：</div>
          <div v-if="exportWorking || showProgress" class="text-body-2 break-all mb-3">
            {{ currentFile }}
          </div>

          <!-- 导出结果 -->
          <div
            v-if="exportResult.success && !exportWorking"
            class="mt-4 p-3 bg-green-50 border border-green-200 rounded"
          >
            <div class="flex items-center justify-between mb-2">
              <div class="text-body-2 text-green-700 font-medium">导出成功</div>
              <v-btn
                icon="mdi-close"
                variant="text"
                size="small"
                @click="handleResetExport"
              ></v-btn>
            </div>
            <div class="text-body-2 mb-2">导出的文件：</div>
            <div
              v-for="(file, index) in exportResult.files"
              :key="index"
              class="text-body-2 mb-1 break-all"
            >
              {{ file }}
            </div>
          </div>

          <div
            v-else-if="exportResult.error && !exportWorking"
            class="mt-4 p-3 bg-red-50 border border-red-200 rounded"
          >
            <div class="flex items-center justify-between mb-2">
              <div class="text-body-2 text-red-700 font-medium">导出失败</div>
              <v-btn
                icon="mdi-close"
                variant="text"
                size="small"
                @click="handleResetExport"
              ></v-btn>
            </div>
            <div class="text-body-2 break-all">{{ exportResult.error }}</div>
          </div>
        </div>
      </v-card>
    </div>
  </div>
</template>
