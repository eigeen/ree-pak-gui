<template>
  <div class="path-scanner">
    <v-card class="mb-4" variant="outlined">
      <v-card-title class="d-flex align-center">
        <v-icon icon="mdi-folder-search" class="mr-3"></v-icon>
        路径扫描
      </v-card-title>
      <v-card-subtitle> 扫描Pak文件和内存转储文件中的路径信息 </v-card-subtitle>
    </v-card>

    <v-container>
      <v-row>
        <!-- Pak 文件选择 -->
        <v-col cols="12" md="6">
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

        <!-- 内存转储文件选择 -->
        <v-col cols="12" md="6">
          <v-card outlined>
            <v-card-title class="text-subtitle-1">内存转储文件</v-card-title>
            <v-card-text>
              <v-btn
                color="primary"
                block
                @click="selectDumpFiles"
                :disabled="scanning"
                class="mb-3"
              >
                选择转储文件
              </v-btn>
              <v-list density="compact" max-height="200" style="overflow-y: auto">
                <v-list-item v-for="(file, index) in dumpFiles" :key="index" class="text-caption">
                  <v-list-item-title>{{ file }}</v-list-item-title>
                  <template v-slot:append>
                    <v-btn
                      icon="mdi-close"
                      size="x-small"
                      @click="removeDumpFile(index)"
                      :disabled="scanning"
                    />
                  </template>
                </v-list-item>
              </v-list>
            </v-card-text>
          </v-card>
        </v-col>
      </v-row>

      <!-- 名称表选择 -->
      <v-row class="mt-4">
        <v-col cols="12">
          <v-card outlined>
            <v-card-title class="text-subtitle-1">名称表（可选）</v-card-title>
            <v-card-text>
              <v-row align="center">
                <v-col cols="8">
                  <v-text-field
                    v-model="nameTableFile"
                    label="名称表文件路径"
                    readonly
                    density="compact"
                    :disabled="scanning"
                  />
                </v-col>
                <v-col cols="4">
                  <v-btn color="primary" @click="selectNameTableFile" :disabled="scanning" block>
                    选择文件
                  </v-btn>
                </v-col>
              </v-row>
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
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { PathScanner } from '@/lib/pathScanner'
import { scanPaths, terminatePathScan, type PathScanOptions, type PathScanProgressEvent } from '@/api/tauri/tools'
import { ShowError, ShowInfo } from '@/utils/message'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { Channel } from '@tauri-apps/api/core'

// 文件选择状态
const pakFiles = ref<string[]>([])
const dumpFiles = ref<string[]>([])
const nameTableFile = ref<string>('')

// 扫描状态
const scanning = ref(false)
const progressMessage = ref('')
const scanResult = ref<string[] | null>(null)

let pathScanner: PathScanner | null = null

// 计算是否可以开始扫描
const canStartScan = computed(() => {
  return pakFiles.value.length > 0 || dumpFiles.value.length > 0
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

const selectDumpFiles = async () => {
  try {
    const selected = await openDialog({
      multiple: true,
      filters: [
        {
          name: 'Dump Files',
          extensions: ['dmp', 'dump', 'bin']
        }
      ]
    })

    if (selected && Array.isArray(selected)) {
      dumpFiles.value = [...dumpFiles.value, ...selected]
    } else if (selected && typeof selected === 'string') {
      dumpFiles.value.push(selected)
    }
  } catch (error) {
    ShowError(`选择文件失败: ${error}`)
  }
}

const selectNameTableFile = async () => {
  try {
    const selected = await openDialog({
      multiple: false,
      filters: [
        {
          name: 'Name Table Files',
          extensions: ['txt', 'nam', 'json']
        }
      ]
    })

    if (selected && typeof selected === 'string') {
      nameTableFile.value = selected
    }
  } catch (error) {
    ShowError(`选择文件失败: ${error}`)
  }
}

// 移除文件
const removePakFile = (index: number) => {
  pakFiles.value.splice(index, 1)
}

const removeDumpFile = (index: number) => {
  dumpFiles.value.splice(index, 1)
}

// 扫描操作
const startScan = async () => {
  if (!canStartScan.value) return

  resetState()
  scanning.value = true

  const options: PathScanOptions = {
    pakFiles: pakFiles.value,
    dumpFiles: dumpFiles.value
  }

  try {
    // pathScanner = new PathScanner(
    //   (event) => {
    //     console.log('event', event)
    //     switch (event.event) {
    //       case 'startFile':
    //         let data = event.data
    //         progressMessage.value = `Scanning file ${data.current} / ${data.total}`
    //         break
    //       case 'scanItem':
    //         data = event.data
    //         progress.value = (data.current / data.total) * 100
    //         break
    //       case 'finish':
    //         if (event.data.success) {
    //           progress.value = 100
    //           scanResult.value = event.data.found_paths
    //         } else {
    //           progressMessage.value = event.data.error ?? 'Unknown Error'
    //         }
    //         scanning.value = false
    //         break
    //     }
    //   },
    // )

    // await pathScanner.scan(options)

    const channel = new Channel<PathScanProgressEvent>()
    channel.onmessage = (event) => {
      console.log('event', event)
      switch (event.event) {
        case 'startFile':
          let data = event.data
          progressMessage.value = `Scanning file ${data.current} / ${data.total}`
          break
        case 'finish':
          if (event.data.success) {
            scanResult.value = event.data.foundPaths
            progressMessage.value = 'Scan finished'
            console.log("scanResult", scanResult.value)
          } else {
            progressMessage.value = event.data.error ?? 'Unknown Error'
          }
          scanning.value = false
          break
      }
    }

    await scanPaths(options, channel)

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
</script>

<style scoped lang="scss">
.path-scanner {
  height: 100%;
  overflow-y: auto;
}

.v-list {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  border-radius: 4px;
}
</style>
