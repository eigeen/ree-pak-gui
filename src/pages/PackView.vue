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
            @click="handleAddFiles"
          >
            添加文件
          </v-btn>
          <v-btn
            class="text-none"
            color="primary"
            prepend-icon="mdi-folder-plus"
            @click="handleAddFolders"
          >
            添加文件夹
          </v-btn>
        </div>
        
        <!-- 文件列表 -->
        <div class="text-subtitle-1 mb-4">文件列表</div>
        <div class="h-[calc(100vh-280px)] overflow-auto">
          <div v-if="inputFiles.length === 0" class="flex flex-col items-center justify-center h-full text-center">
            <v-icon icon="mdi-file-outline" size="64" color="grey-lighten-1" class="mb-4"></v-icon>
            <p class="text-grey-lighten-1 text-h6">尚未添加文件</p>
            <p class="text-grey-lighten-1 text-body-2">点击上方按钮添加文件或文件夹</p>
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
            <v-radio
              label="每个文件单独导出 pak"
              value="individual"
              density="compact"
            ></v-radio>
            <v-radio
              label="所有文件导出为单个 pak"
              value="single"
              density="compact"
            ></v-radio>
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

        <!-- 导出目录 -->
        <div class="mb-4">
          <div class="text-body-2 mb-2">导出目录</div>
          <div class="flex gap-2">
            <v-text-field
              v-model="exportDirectory"
              variant="outlined"
              density="comfortable"
              hide-details
              readonly
              placeholder="选择目录"
            ></v-text-field>
            <v-btn
              icon="mdi-folder-open"
              variant="outlined"
              @click="handleSelectDirectory"
            ></v-btn>
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
      </v-card>
    </div>
  </div>

  <!-- 导出进度对话框 -->
  <v-dialog v-model="showProgressDialog" persistent max-width="500">
    <v-card>
      <v-card-text class="pa-8">
        <div class="text-center text-h6 mb-4">
          正在导出文件...
          <span v-if="!exportWorking">完成！</span>
        </div>
        <v-progress-linear
          :color="progressValue >= 100 ? 'green' : 'primary'"
          height="12px"
          :model-value="progressValue"
          rounded
          class="mb-2"
        ></v-progress-linear>
        <div class="text-body-1 mb-4">
          {{ finishFileCount }} / {{ totalFileCount }} 个文件
        </div>
        <div class="text-body-2">正在导出：</div>
        <div class="text-body-2">{{ currentFile }}</div>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          class="text-none"
          :color="exportWorking ? 'error' : 'primary'"
          @click="handleCloseProgress"
        >
          {{ exportWorking ? '终止' : '关闭' }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- 导出结果对话框 -->
  <v-dialog v-model="showResultDialog" max-width="600">
    <v-card>
      <v-card-title class="text-h6">
        导出结果
      </v-card-title>
      <v-card-text>
        <div v-if="exportResult.success">
          <div class="text-body-1 mb-4 text-green-600">
            导出成功
          </div>
          <div class="text-body-2 mb-2">导出的文件：</div>
          <v-list density="compact">
            <v-list-item
              v-for="(file, index) in exportResult.files"
              :key="index"
              class="text-body-2"
            >
              <v-list-item-title>{{ file }}</v-list-item-title>
            </v-list-item>
          </v-list>
        </div>
        <div v-else>
          <div class="text-body-1 mb-4 text-red-600">
            导出失败
          </div>
          <div class="text-body-2">{{ exportResult.error }}</div>
        </div>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn
          class="text-none"
          color="primary"
          @click="showResultDialog = false"
        >
          关闭
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

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
const showProgressDialog = ref(false)
const currentFile = ref('')
const totalFileCount = ref(0)
const finishFileCount = ref(0)
const progressValue = computed(() =>
  totalFileCount.value === 0 ? 0 : (finishFileCount.value / totalFileCount.value) * 100
)

// 导出结果
const showResultDialog = ref(false)
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

// 处理添加文件
const handleAddFiles = () => {
  console.log('添加文件')
  // TODO: 实现文件选择对话框
}

// 处理添加文件夹
const handleAddFolders = () => {
  console.log('添加文件夹')
  // TODO: 实现文件夹选择对话框
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
  
  // 模拟导出进度
  exportWorking.value = true
  showProgressDialog.value = true
  totalFileCount.value = inputFiles.value.length
  finishFileCount.value = 0
  
  // 模拟进度更新
  const interval = setInterval(() => {
    finishFileCount.value++
    currentFile.value = `file_${finishFileCount.value}.pak`
    
    if (finishFileCount.value >= totalFileCount.value) {
      clearInterval(interval)
      exportWorking.value = false
      setTimeout(() => {
        showProgressDialog.value = false
        showResultDialog.value = true
        exportResult.value = {
          success: true,
          files: ['output1.pak', 'output2.pak'],
          error: ''
        }
      }, 1000)
    }
  }, 500)
}

// 处理关闭进度对话框
const handleCloseProgress = () => {
  if (exportWorking.value) {
    console.log('终止导出')
    // TODO: 实现终止导出功能
    exportWorking.value = false
  }
  showProgressDialog.value = false
}
</script> 