<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, toRefs } from 'vue'
import { ShowError, ShowWarn } from '@/utils/message'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { exists, stat } from '@tauri-apps/plugin-fs'
import FileConflict, { type ConflictFile } from '@/components/FileConflict.vue'
import { useWorkStore, type FileItem } from '@/store/work'

const workStore = useWorkStore()

const { exportConfig, inputFiles } = toRefs(workStore.pack)

// 导出进度
const exportWorking = ref(false)
const currentFile = ref('')
const totalFileCount = ref(0)
const finishFileCount = ref(0)
const progressValue = computed(() =>
  totalFileCount.value === 0 ? 0 : (finishFileCount.value / totalFileCount.value) * 100
)

// 导出结果
const exportResult = ref({
  success: false,
  files: [] as string[],
  error: ''
})

// 冲突处理
const conflictDialogVisible = ref(false)
const conflictFiles = ref<ConflictFile[]>([])
const conflictResolutions = ref<{ [relativePath: string]: number }>({})

// 计算属性
const enableExport = computed(() => {
  return inputFiles.value.length > 0 && exportConfig.value.exportDirectory !== ''
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
        isPak: st.isFile && path.endsWith('.pak')
      })

      // fast mode
      if (exportConfig.value.fastMode) {
        await handleExport()
      }
    }

    inputFiles.value.push(...addList)
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
  console.debug('handleExport', {
    files: inputFiles.value,
    mode: exportConfig.value.mode,
    autoDetectRoot: exportConfig.value.autoDetectRoot,
    exportDirectory: exportConfig.value.exportDirectory
  })

  // 重置导出状态
  exportResult.value = {
    success: false,
    files: [],
    error: ''
  }

  // 模拟检测文件冲突
  const conflicts = simulateConflictDetection()

  if (conflicts.length > 0) {
    // 发现冲突，显示冲突处理对话框
    conflictFiles.value = conflicts
    conflictDialogVisible.value = true
  } else {
    // 没有冲突，直接导出
    await proceedWithExport()
  }
}

// 处理重置导出状态
const handleResetExport = () => {
  exportWorking.value = false
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
  await addFiles(paths)
}

// 冲突处理方法
const handleConflictResolve = () => {
  // 从 conflictFiles 中提取解决方案
  const resolutions: { [relativePath: string]: number } = {}
  conflictFiles.value.forEach((conflict) => {
    resolutions[conflict.relativePath] = conflict.selectedSource
  })

  conflictResolutions.value = resolutions
  conflictDialogVisible.value = false

  // 继续导出过程
  proceedWithExport()
}

const handleConflictCancel = () => {
  conflictDialogVisible.value = false
  // 重置导出状态
  handleResetExport()
}

const proceedWithExport = async () => {
  console.debug('继续导出过程，冲突解决方案:', conflictResolutions.value)

  // 这里实现实际的导出逻辑
  // 使用 conflictResolutions.value 来决定选择哪个文件源

  // 模拟导出过程
  exportWorking.value = true
  totalFileCount.value = inputFiles.value.length
  finishFileCount.value = 0

  // 模拟进度更新
  const interval = setInterval(() => {
    finishFileCount.value++
    currentFile.value = `processing_${finishFileCount.value}.pak`

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

const simulateConflictDetection = () => {
  // 模拟检测到文件冲突
  const mockConflicts: ConflictFile[] = [
    {
      relativePath:
        '/natives/STM/streaming/Art/Model/Character/ch03/002/001/2/textures/ch03_002_0012_ALBD.tex',
      size: 1048576,
      modifiedDate: new Date('2024-01-20T14:45:00'),
      sources: [
        {
          sourcePath:
            'C:/mod1/natives/STM/streaming/Art/Model/Character/ch03/002/001/2/textures/ch03_002_0012_ALBD.tex'
        },
        {
          sourcePath:
            'C:/mod2/natives/STM/streaming/Art/Model/Character/ch03/002/001/2/textures/ch03_002_0012_ALBD.tex'
        }
      ],
      selectedSource: 1
    },
    {
      relativePath: '/natives/STM/streaming/Art/Model/Character/ch01/001/texture.tex',
      size: 524288,
      modifiedDate: new Date('2024-02-01T11:30:00'),
      sources: [
        {
          sourcePath: 'C:/mod1/natives/STM/streaming/Art/Model/Character/ch01/001/texture.tex'
        },
        {
          sourcePath: 'C:/mod3/natives/STM/streaming/Art/Model/Character/ch01/001/texture.tex'
        },
        {
          sourcePath: 'C:/mod4/natives/STM/streaming/Art/Model/Character/ch01/001/texture.tex'
        }
      ],
      selectedSource: 2
    }
  ]

  return mockConflicts
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
          <v-btn
            class="text-none"
            color="warning"
            prepend-icon="mdi-alert-circle"
            @click="() => {
              conflictDialogVisible = true
              conflictFiles = simulateConflictDetection()
            }"
          >
            测试冲突
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
          class="text-none"
          color="primary"
          prepend-icon="mdi-export"
          @click="handleExport"
          :disabled="!enableExport"
          block
        >
          导出
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
