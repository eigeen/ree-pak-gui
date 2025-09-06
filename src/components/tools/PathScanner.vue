<template>
  <div class="path-scanner">
    <!-- 中置容器 -->
    <div class="content-container mx-auto">
      <!-- 标题 -->
      <div class="text-h5 mb-4">路径扫描</div>

      <!-- 描述栏 -->
      <div class="text-body-1 mb-2">尝试扫描Pak文件内包含的子文件路径。</div>
      <v-alert color="info" variant="tonal" class="mb-2">
        <div class="text-body-2">
          <v-icon size="small" class="mr-1">mdi-information</v-icon>
          这是一个轻量版，如果需要扫描较大文件，请使用
          <a
            href="#"
            @click="openUrl('https://github.com/eigeen/ree-path-searcher')"
            class="text-primary text-decoration-none"
          >
            独立版本
          </a>
          以获得更高性能
        </div>
      </v-alert>

      <v-container>
        <v-row>
          <!-- Pak 文件选择 -->
          <v-col cols="12">
            <v-card outlined>
              <v-card-title class="text-subtitle-1">Pak 文件</v-card-title>
              <v-card-text>
                <v-btn
                  color="primary"
                  block
                  @click="selectPakFiles"
                  :disabled="scanning"
                  class="mb-3"
                >
                  选择 Pak 文件
                </v-btn>
                <v-list density="compact" max-height="200" style="overflow-y: auto">
                  <v-list-item v-for="(file, index) in pakFiles" :key="index" class="text-caption">
                    <v-list-item-title>{{ file }}</v-list-item-title>
                    <template v-slot:append>
                      <v-btn
                        icon="mdi-close"
                        size="x-small"
                        @click="removePakFile(index)"
                        :disabled="scanning"
                      />
                    </template>
                  </v-list-item>
                </v-list>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>

        <!-- 已知路径列表选择 -->
        <v-row class="mt-4">
          <v-col cols="12">
            <v-card outlined>
              <v-card-title class="text-subtitle-1">已知路径列表（可选）</v-card-title>
              <v-card-text>
                <FileNameTableSelector v-model="selectedFileList" :items="comboItems" />
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>

        <!-- 进度显示 -->
        <v-row v-if="scanning || scanResult" class="mt-4">
          <v-col cols="12">
            <v-card outlined>
              <v-card-title class="text-subtitle-1">扫描状态</v-card-title>
              <v-card-text>
                <v-progress-linear
                  v-if="scanning"
                  indeterminate
                  color="primary"
                  height="20"
                  class="mb-3"
                >
                  <template v-slot:default="{ value }">
                    <small class="text-white">{{ Math.ceil(value) }}%</small>
                  </template>
                </v-progress-linear>

                <div v-if="progressMessage" class="text-body-2 text-grey-darken-1 mb-2">
                  {{ progressMessage }}
                </div>

                <!-- 结果显示 -->
                <div v-if="scanResult && !scanning">
                  <div class="text-subtitle-2 mb-2">
                    扫描完成 - 找到 {{ scanResult.length }} 个路径
                  </div>
                  <v-textarea
                    :model-value="scanResult.join('\n')"
                    readonly
                    rows="10"
                    variant="outlined"
                    class="text-caption"
                    hide-details
                  />
                  <v-btn color="primary" @click="copyResults" class="mt-2"> 复制结果 </v-btn>
                </div>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>

        <!-- 操作按钮 -->
        <v-row class="mt-4">
          <v-col cols="12" class="d-flex justify-end gap-3">
            <v-btn v-if="scanning" color="error" @click="stopScan"> 停止扫描 </v-btn>
            <v-btn v-if="!scanning" color="primary" @click="startScan" :disabled="!canStartScan">
              开始扫描
            </v-btn>
          </v-col>
        </v-row>
      </v-container>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { PathScanner } from '@/lib/pathScanner'
import { terminatePathScan, type PathScanOptions } from '@/api/tauri/tools'
import { ShowError, ShowInfo } from '@/utils/message'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import FileNameTableSelector from '@/components/FileNameTable/FileNameTableSelector.vue'
import { useFileListStore } from '@/store/filelist'

// 文件选择状态
const pakFiles = ref<string[]>([])
const selectedFileList = ref<string>('')

// 文件列表状态
const fileListStore = useFileListStore()

// 扫描状态
const scanning = ref(false)
const progressMessage = ref('')
const scanResult = ref<string[] | null>(null)

let pathScanner: PathScanner | null = null

// 计算是否可以开始扫描
const canStartScan = computed(() => {
  return pakFiles.value.length > 0
})

// 重置状态
const resetState = () => {
  progressMessage.value = ''
  scanResult.value = null
}

// 文件选择方法
const selectPakFiles = async () => {
  try {
    const selected = await openDialog({
      multiple: true,
      filters: [
        {
          name: 'Pak Files',
          extensions: ['pak']
        }
      ]
    })

    if (selected && Array.isArray(selected)) {
      pakFiles.value = [...pakFiles.value, ...selected]
    } else if (selected && typeof selected === 'string') {
      pakFiles.value.push(selected)
    }
  } catch (error) {
    ShowError(`选择文件失败: ${error}`)
  }
}

// 移除文件
const removePakFile = (index: number) => {
  pakFiles.value.splice(index, 1)
}

// 扫描操作
const startScan = async () => {
  if (!canStartScan.value) return

  resetState()
  scanning.value = true

  const options: PathScanOptions = {
    pakFiles: pakFiles.value,
    dumpFiles: []
  }

  try {
    pathScanner = new PathScanner((event) => {
      switch (event.event) {
        case 'startFile':
          let data = event.data
          progressMessage.value = `Scanning file ${data.current} / ${data.total}`
          break
        case 'finish':
          if (event.data.success) {
            scanResult.value = event.data.foundPaths
            progressMessage.value = 'Scan finished'
            console.log('scanResult', scanResult.value)
          } else {
            progressMessage.value = event.data.error ?? 'Unknown Error'
          }
          scanning.value = false
          break
      }
    })
    await pathScanner.scan(options)
  } catch (error) {
    scanning.value = false
    ShowError(`扫描失败: ${error}`)
  }
}

const stopScan = async () => {
  if (pathScanner) {
    // await pathScanner.terminate()
    await terminatePathScan()
  }
  scanning.value = false
  progressMessage.value = '扫描已停止'
}

const copyResults = async () => {
  if (!scanResult.value || scanResult.value.length === 0) {
    return
  }

  try {
    await navigator.clipboard.writeText(scanResult.value.join('\n'))
    ShowInfo('结果已复制到剪贴板')
  } catch (error) {
    ShowError(`复制失败: ${error}`)
  }
}

// 获取文件列表选项
const localSources = computed(() => {
  const itemsMap: { [identifier: string]: any } = {}
  for (const identifier in fileListStore.localFile) {
    itemsMap[identifier] = {
      ...fileListStore.localFile[identifier].source
    }
  }
  for (const fileName in fileListStore.downloadedFile) {
    const source = fileListStore.downloadedFile[fileName].source
    const identifier = source.identifier
    if (identifier in itemsMap) {
      continue
    }
    itemsMap[identifier] = {
      ...source
    }
  }

  const sources = Object.values(itemsMap)
  sources.sort((a: any, b: any) => a.identifier.localeCompare(b.identifier))
  return sources
})

const comboItems = computed(() =>
  localSources.value.map((item: any) => {
    return { label: item.identifier, value: item.identifier }
  })
)

// 初始化文件列表
onMounted(async () => {
  try {
    await fileListStore.refreshLocalSource()
  } catch (error) {
    console.error('Failed to load file list:', error)
  }
})
</script>

<style scoped lang="scss">
.path-scanner {
  height: 100%;
  overflow-y: auto;
  padding: 0 2rem;
}

.content-container {
  max-width: 600px;
  width: 100%;
}

.v-col {
  padding-left: 0;
  padding-right: 0;
}

.v-list {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  border-radius: 4px;
}
</style>
