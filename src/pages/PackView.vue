<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, toRefs } from 'vue'
import { ShowError, ShowWarn } from '@/utils/message'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { exists, stat } from '@tauri-apps/plugin-fs'
import FileConflict from '@/components/FileConflict.vue'
import HoverBubble from '@/components/HoverBubble.vue'
import { useWorkStore, type FileItem } from '@/store/work'
import {
  Packer,
  type ConflictFile,
  type ExportConfig,
  type ExportResult,
  type PackProgress
} from '@/lib/packer'

const workStore = useWorkStore()

const { exportConfig, inputFiles } = toRefs(workStore.pack)

// 创建Packer实例
const packer = new Packer(
  (progress: PackProgress) => {
    // 进度更新回调
    exportWorking.value = progress.working
    currentFile.value = progress.currentFile
    totalFileCount.value = progress.totalFileCount
    finishFileCount.value = progress.finishFileCount
    progressValue.value = progress.progressValue
  },
  (result: ExportResult) => {
    // 结果更新回调
    exportResult.value = result
  }
)

// 导出进度
const exportWorking = ref(false)
const currentFile = ref('')
const totalFileCount = ref(0)
const finishFileCount = ref(0)
const progressValue = ref(0)

// 导出结果
const exportResult = ref<ExportResult>({
  success: false,
  files: [],
  error: ''
})

// 冲突处理
const conflictDialogVisible = ref(false)
const conflictFiles = ref<ConflictFile[]>([])

// 计算属性
const enableExport = computed(() => {
  if (inputFiles.value.length === 0) return false

  if (exportConfig.value.mode === 'single' && !exportConfig.value.exportDirectory) {
    return false
  }

  return true
})

// Add files or folders
const addFiles = async (paths: string[]) => {
  try {
    const addList: FileItem[] = []

    for (const path of paths) {
      if (!(await exists(path))) {
        ShowError(`输入文件 ${path} 不存在。`)
        continue
      }

      const st = await stat(path)
      addList.push({
        path,
        isFile: st.isFile
      })
    }

    inputFiles.value.push(...addList)

    // fast mode
    if (exportConfig.value.fastMode) {
      await handleExport()
    }
  } catch (e) {
    ShowError(e)
  }
}

// 处理添加文件
const handleAddViaDialog = async (pak: boolean) => {
  const results = await openDialog({
    multiple: true,
    directory: !pak,
    filters: pak ? [{ name: 'Pak Files', extensions: ['pak'] }] : undefined
  })
  if (!results) return

  await addFiles(results)
}

const handleCloseAll = () => {
  inputFiles.value = []
}

// 处理选择导出目录
const handleSelectDirectory = async () => {
  const result = await openDialog({
    directory: true
  })
  if (!result) return

  exportConfig.value.exportDirectory = result
}

// 处理移除文件
const handleRemoveFile = (index: number) => {
  console.log('移除文件', index)
  inputFiles.value.splice(index, 1)
}

// 处理导出
const handleExport = async () => {
  await packer.handleExport(inputFiles.value, exportConfig.value)
}

// 终止导出操作
const handleTerminateExport = async () => {
  await packer.terminateExport()
}

// 处理重置导出状态
const handleResetExport = () => {
  packer.resetExport()
}

const dropInAddFiles = async (paths: string[]) => {
  await addFiles(paths)
}

// 冲突处理方法
const handleConflictResolve = () => {
  // 从 conflictFiles 中提取解决方案
  const resolutions: { [relativePath: string]: number } = {}
  conflictFiles.value.forEach((conflict) => {
    resolutions[conflict.relativePath] = conflict.selectedSource
  })

  packer.setConflictResolutions(resolutions)
  conflictDialogVisible.value = false

  // 继续导出过程
  packer.proceedWithMergeExport(inputFiles.value, exportConfig.value)
}

const handleConflictCancel = () => {
  conflictDialogVisible.value = false
  // 重置导出状态
  handleResetExport()
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
            prepend-icon="mdi-folder-plus"
            @click="handleAddViaDialog(false)"
          >
            添加文件夹
          </v-btn>
          <v-tooltip text="可用于合并多个 Pak 文件" location="top">
            <template #activator="{ props }">
              <v-btn
                v-bind="props"
                class="text-none"
                color="primary"
                prepend-icon="mdi-file-plus"
                @click="handleAddViaDialog(true)"
              >
                添加 Pak
              </v-btn>
            </template>
          </v-tooltip>
          <v-btn class="text-none" prepend-icon="mdi-close-box-multiple" @click="handleCloseAll">
            移除全部
          </v-btn>
        </div>

        <!-- 文件列表 -->
        <div class="text-subtitle-1">文件列表</div>
        <div class="h-[calc(100vh-230px)] overflow-auto">
          <!-- 空内容提示 -->
          <div
            v-if="inputFiles.length === 0"
            class="flex flex-col items-center justify-center h-full text-center"
          >
            <v-icon
              icon="mdi-folder-outline"
              size="64"
              color="grey-lighten-1"
              class="mb-4"
            ></v-icon>
            <p class="text-grey-lighten-1 text-h6">尚未添加文件</p>
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
                  <v-icon icon="mdi-folder" color="blue"></v-icon>
                </template>

                {{ file.path }}

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
          <v-radio-group v-model="exportConfig.mode" density="compact" hide-details>
            <v-radio label="每个文件项单独导出 pak" value="individual" density="compact"></v-radio>
            <v-radio label="所有文件导出为单个 pak" value="single" density="compact"></v-radio>
          </v-radio-group>
        </div>

        <!-- 导出配置 -->
        <div class="mb-4">
          <div class="flex items-center gap-2">
            <v-checkbox
              v-model="exportConfig.autoDetectRoot"
              label="自动检测根目录"
              density="compact"
              color="primary"
              hide-details
            ></v-checkbox>
            <HoverBubble>自动检测第一个 natives/STM/** 路径作为根目录</HoverBubble>
          </div>

          <div class="flex items-center gap-2">
            <v-checkbox
              v-model="exportConfig.fastMode"
              label="快速模式"
              density="compact"
              color="primary"
              hide-details
            ></v-checkbox>
            <HoverBubble>
              导入文件后会自动导出到指定目录，无需确认。<br />
              如未指定目录，则导出到输入文件相同目录。
            </HoverBubble>
          </div>
        </div>

        <!-- 导出文件 -->
        <div class="mb-4">
          <div class="text-body-2 mb-2">导出文件</div>
          <div class="flex gap-2">
            <v-text-field
              v-model="exportConfig.exportDirectory"
              variant="outlined"
              density="comfortable"
              hide-details
              placeholder="导出目录"
            ></v-text-field>
            <v-btn icon="mdi-folder-open" variant="text" @click="handleSelectDirectory"></v-btn>
          </div>
        </div>

        <!-- 导出按钮 -->
        <v-btn
          v-if="!exportWorking"
          class="text-none"
          color="primary"
          prepend-icon="mdi-export"
          @click="handleExport"
          :disabled="!enableExport"
          block
        >
          导出
        </v-btn>

        <!-- 取消导出按钮 -->
        <v-btn
          v-if="exportWorking"
          class="text-none"
          color="warning"
          prepend-icon="mdi-stop"
          @click="handleTerminateExport"
          block
        >
          取消导出
        </v-btn>

        <!-- 导出报告信息 -->
        <div v-if="exportWorking || exportResult.success || exportResult.error" class="mt-4">
          <v-progress-linear
            v-if="progressValue > 0"
            :color="progressValue >= 100 ? 'green' : 'primary'"
            height="12px"
            :model-value="progressValue"
            rounded
            class="mb-2"
          ></v-progress-linear>

          <!-- 进度信息 -->
          <div v-if="progressValue > 0" class="text-body-2 mb-2">
            {{ finishFileCount }} / {{ totalFileCount }} 个文件
          </div>
          <div v-if="progressValue > 0" class="text-body-2 mb-1">正在导出：</div>
          <div v-if="progressValue > 0" class="text-body-2 break-all mb-3">
            {{ currentFile }}
          </div>

          <!-- 导出结果 -->
          <div
            v-if="exportResult.success && !exportWorking"
            class="mt-4 pa-2 bg-green-50 border border-green-200 rounded"
          >
            <div class="flex items-center justify-between">
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
            class="mt-4 pa-2 bg-red-50 border border-red-200 rounded"
          >
            <div class="flex items-center justify-between">
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

    <!-- 文件冲突处理对话框 -->
    <v-dialog v-model="conflictDialogVisible" max-width="800px" persistent>
      <v-card>
        <v-card-title class="text-h6 pa-4">
          <v-icon icon="mdi-alert-circle" color="warning" class="mr-2"></v-icon>
          处理文件冲突
        </v-card-title>

        <v-card-text class="pa-4">
          <FileConflict v-model:conflicts="conflictFiles" />
        </v-card-text>

        <v-card-actions class="pa-4">
          <v-spacer></v-spacer>
          <v-btn color="grey" variant="text" @click="handleConflictCancel">取消</v-btn>
          <v-btn color="primary" @click="handleConflictResolve">确定</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>
