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

import { useI18n } from 'vue-i18n'
const { t } = useI18n()

const workStore = useWorkStore()

const { exportConfig, inputFiles } = toRefs(workStore.pack)

// 创建Packer实例
const packer = new Packer(
  (p: PackProgress) => {
    // 进度更新回调
    if (p.totalFileCount === p.finishFileCount) {
      // 导出完成
      p.currentFile = ''
    }
    progress.value = p
  },
  (result: ExportResult) => {
    // 结果更新回调
    exportResult.value = result
  }
)

// 导出进度
const progress = ref<PackProgress>({
  working: false,
  currentFile: '',
  totalFileCount: 0,
  finishFileCount: 0
})
const progressValue = computed(() => {
  if (progress.value.totalFileCount === 0) return 0
  return (progress.value.finishFileCount / progress.value.totalFileCount) * 100
})
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
        ShowError(`Input file ${path} does not exist.`)
        continue
      }

      // check if exists in inputFiles
      if (inputFiles.value.some((f) => f.path === path)) {
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
      handleCloseAll()
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
  console.log('Remove file', index)
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
            {{ t('pack.addFolder') }}
          </v-btn>
          <v-tooltip :text="t('pack.addPakTooltip')" location="top">
            <template #activator="{ props }">
              <v-btn
                v-bind="props"
                class="text-none"
                color="primary"
                prepend-icon="mdi-file-plus"
                @click="handleAddViaDialog(true)"
              >
                {{ t('pack.addPak') }}
              </v-btn>
            </template>
          </v-tooltip>
          <v-btn class="text-none" prepend-icon="mdi-close-box-multiple" @click="handleCloseAll">
            {{ t('pack.removeAll') }}
          </v-btn>
        </div>

        <!-- 文件列表 -->
        <div class="text-subtitle-1">{{ t('pack.fileList') }}</div>
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
            <p class="text-grey-lighten-1 text-h6">{{ t('pack.noFilesAdded') }}</p>
            <p class="text-grey-lighten-1 text-body-2">{{ t('pack.noFilesAddedDesc') }}</p>
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
        <div class="text-subtitle-1 mb-4">{{ t('pack.exportSettings') }}</div>

        <!-- 导出模式 -->
        <div class="mb-4">
          <div class="text-body-2 mb-2">{{ t('pack.exportMode') }}</div>
          <v-radio-group v-model="exportConfig.mode" density="compact" hide-details>
            <v-radio :label="t('pack.exportModeIndividual')" value="individual" density="compact"></v-radio>
            <!-- <v-radio :label="t('pack.exportModeSingle')" value="single" density="compact"></v-radio> -->
          </v-radio-group>
        </div>

        <!-- 导出配置 -->
        <div class="mb-4">
          <div class="flex items-center gap-2">
            <v-checkbox
              v-model="exportConfig.autoDetectRoot"
              :label="t('pack.autoDetectRoot')"
              density="compact"
              color="primary"
              hide-details
            ></v-checkbox>
            <HoverBubble>{{ t('pack.autoDetectRootTooltip') }}</HoverBubble>
          </div>

          <div class="flex items-center gap-2">
            <v-checkbox
              v-model="exportConfig.fastMode"
              :label="t('pack.fastMode')"
              density="compact"
              color="primary"
              hide-details
            ></v-checkbox>
            <HoverBubble>
              {{ t('pack.fastModeTooltipL1') }}<br />
              {{ t('pack.fastModeTooltipL2') }}
            </HoverBubble>
          </div>
        </div>

        <!-- 导出文件 -->
        <div class="mb-4">
          <div class="text-body-2 mb-2">{{ t('pack.exportDirectory') }}</div>
          <div class="flex gap-2">
            <v-text-field
              v-model="exportConfig.exportDirectory"
              variant="outlined"
              density="comfortable"
              hide-details
              :placeholder="t('pack.exportDirectoryPlaceholder')"
            ></v-text-field>
            <v-btn icon="mdi-folder-open" variant="text" @click="handleSelectDirectory"></v-btn>
          </div>
        </div>

        <!-- 导出按钮 -->
        <v-btn
          v-if="!progress.working"
          class="text-none"
          color="primary"
          prepend-icon="mdi-export"
          @click="handleExport"
          :disabled="!enableExport"
          block
        >
          {{ t('pack.export') }}
        </v-btn>

        <!-- 取消导出按钮 -->
        <v-btn
          v-if="progress.working"
          class="text-none"
          color="warning"
          prepend-icon="mdi-stop"
          @click="handleTerminateExport"
          block
        >
          {{ t('pack.cancelExport') }}
        </v-btn>

        <!-- 导出报告信息 -->
        <div v-if="progress.working || exportResult.success || exportResult.error" class="mt-4">
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
            {{ progress.finishFileCount }} / {{ progress.totalFileCount }} {{ t('pack.filesCount') }}
          </div>
          <div v-if="progress.currentFile" class="text-body-2 mb-1">{{ t('pack.exporting') }}</div>
          <div v-if="progress.currentFile" class="text-body-2 break-all mb-3">
            {{ progress.currentFile }}
          </div>

          <!-- 导出结果 -->
          <div v-if="exportResult.success && !progress.working" class="mt-4">
            <div class="text-body-2 text-green-700 font-medium mb-2">
              <v-icon icon="mdi-check-circle" color="green" size="small" class="mr-1"></v-icon>
              {{ t('pack.exportSuccess') }}
            </div>
            
            <!-- 文件树显示 -->
            <div v-if="exportResult.fileTree" class="mt-3">
              <div class="text-body-2 font-medium mb-2">{{ t('pack.fileStructure') }}</div>
              <pre class="text-xs bg-gray-50 p-2 rounded border max-h-48 max-w-full overflow-auto font-mono whitespace-pre">{{ exportResult.fileTree }}</pre>
            </div>
          </div>

          <div v-else-if="exportResult.error && !progress.working" class="mt-4">
            <div class="text-body-2 text-red-700 font-medium mb-2">
              <v-icon icon="mdi-alert-circle" color="red" size="small" class="mr-1"></v-icon>
              {{ t('pack.exportFailed') }}
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
          {{ t('pack.fileConflictTitle') }}
        </v-card-title>

        <v-card-text class="pa-4">
          <FileConflict v-model:conflicts="conflictFiles" />
        </v-card-text>

        <v-card-actions class="pa-4">
          <v-spacer></v-spacer>
          <v-btn color="grey" variant="text" @click="handleConflictCancel">{{ t('pack.cancel') }}</v-btn>
          <v-btn color="primary" @click="handleConflictResolve">{{ t('pack.confirm') }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>
