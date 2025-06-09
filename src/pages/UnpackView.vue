<template>
  <el-container class="main-container">
    <el-aside class="aside-outer">
      <div class="aside-container">
        <v-card class="pa-4 elevation-3 rounded-lg tool-chunk">
          <div class="text-subtitle-1">File List</div>
          <FileNameTable v-model="workStore.unpack.fileList"></FileNameTable>
        </v-card>
        <v-card class="pa-4 elevation-3 rounded-lg tool-chunk">
          <div class="text-subtitle-1">Pak Files</div>
          <PakFiles
            :pak-list="pakData"
            :enable-add="enableAddPaks"
            @open="handleOpen"
            @close="handleClose"
            @order="handleOrder"
            @close-all="handleCloseAll"
          ></PakFiles>
        </v-card>
        <v-card class="pa-4 elevation-3 rounded-lg tool-chunk">
          <v-text-field
            v-model="workStore.unpack.filterText"
            variant="outlined"
            density="comfortable"
            hide-details
            label="Filter keyword"
          ></v-text-field>
          <v-checkbox
            v-model="workStore.unpack.filterUseRegex"
            label="Regex"
            density="compact"
            color="primary"
            hide-details
          ></v-checkbox>
          <v-btn
            class="text-none"
            prepend-icon="mdi-filter-variant"
            :disabled="workStore.unpack.filterText === filterTextApply"
            @click="updateFilter"
            >Apply Filter</v-btn
          >
        </v-card>
      </div>
    </el-aside>

    <div class="main-content">
      <v-card class="pa-2 elevation-3 rounded-lg tree-card">
        <!-- 渲染确认覆盖层 -->
        <div v-if="showOverlay" class="overlay" @click.stop>
          <div class="overlay-content">
            <v-btn
              :disabled="loadingTree"
              :loading="loadingTree"
              class="load-btn text-none"
              color="primary"
              @click="doRender"
            >
              <v-icon icon="mdi-refresh" class="mr-2"></v-icon>
              Load File Tree
            </v-btn>
          </div>
        </div>
        <div class="tree-panel">
          <FileTree
            class="file-tree"
            ref="fileTreeComponent"
            :data="treeData"
            :filter-text="filterTextApply"
            :regex-mode="workStore.unpack.filterUseRegex"
          ></FileTree>
          <div class="tree-actions">
            <v-btn
              class="text-none"
              color="primary"
              prepend-icon="mdi-export"
              @click="doExtraction"
              :disabled="!enableExtract"
              >Extract</v-btn
            >
          </div>
        </div>
      </v-card>
    </div>
  </el-container>

  <v-dialog v-model="showProgressPanel" persistent>
    <v-card>
      <v-card-text class="pa-8">
        <div class="text-center text-h6 mb-4">
          Extracting Files... <span v-if="!unpackWorking">Done!</span>
        </div>
        <v-progress-linear
          :color="progressValue >= 100 ? 'green' : 'primary'"
          height="12px"
          :model-value="progressValue"
          rounded
          class="mb-2"
        ></v-progress-linear>
        <div class="text-body-1 mb-4">{{ finishFileCount }} / {{ totalFileCount }} files</div>
        <div class="text-body-2">Extracting:</div>
        <div class="text-body-2">{{ currentFile }}</div>
      </v-card-text>
      <div class="progress-actions">
        <v-btn
          class="ma-4 text-none"
          :color="unpackWorking ? 'error' : 'primary'"
          @click="handleCloseProgress"
        >
          {{ unpackWorking ? 'Terminate' : 'Close' }}
        </v-btn>
      </div>
    </v-card>
  </v-dialog>

  <v-dialog v-model="showConfirmTermination" max-width="400" persistent>
    <v-card class="pa-2">
      <v-card-title class="text-h6">Confirm Termination</v-card-title>
      <v-card-text
        >Did you want to terminate the current extraction operation? <br />The extracted files will
        be retained.</v-card-text
      >
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn color="grey" text @click="showConfirmTermination = false">Cancel</v-btn>
        <v-btn color="error" text @click="handleConfirmTermination">Confirm</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { Channel } from '@tauri-apps/api/core'
import { getCurrentWindow, ProgressBarStatus } from '@tauri-apps/api/window'
import { exists } from '@tauri-apps/plugin-fs'

import {
  pak_close,
  pak_extract_all,
  pak_list_all,
  pak_open,
  pak_read_file_tree_optimized,
  pak_terminate_extraction
} from '@/api/tauri/pak'
import type { ExtractOptions, PakInfo, RenderTreeNode, WorkProgressEvent } from '@/api/tauri/pak'
import PakFiles from '@/components/PakFiles.vue'
import FileTree from '@/components/FileTree.vue'
import { file_table_load } from '@/api/tauri/filelist'
import { ShowError, ShowWarn } from '@/utils/message'
import { useWorkStore } from '@/store/work'

const workStore = useWorkStore()

// 过滤器输入（应用输入）
const filterTextApply = ref('')
// 已加载的pak
const pakData = ref<PakInfo[]>([])
const initialLoaded = ref(false)
// 树视图数据
const treeData = ref<RenderTreeNode | null>(null)
// show overlay
const showOverlay = ref(false)
// is tree loading
const loadingTree = ref(false)
// const fileNameTablePath = ref('')
// 解包进度条
const unpackWorking = ref(false)
const showProgressPanel = ref(false)
const currentFile = ref('')
const totalFileCount = ref(0)
const finishFileCount = ref(0)
const progressValue = computed(() =>
  totalFileCount.value === 0 ? 0 : (finishFileCount.value / totalFileCount.value) * 100
)
const showConfirmTermination = ref(false)
// 是否允许添加Pak文件
const enableAddPaks = computed(() => {
  return workStore.unpack.fileList !== ''
})

const fileTreeComponent = ref<InstanceType<typeof FileTree>>()
// const paks = ref<Map<string, PakId>>(new Map());
// const canRenderTree = ref(false);
// // 启用FileList选择
// const enableFileListSelect = computed(() => pakData.value.length > 0);
// 启用树渲染按键
// const enableTreeRender = computed(() => treeData.value ? treeData > 0 : false);
// 启用解压按键
const enableExtract = computed(() => treeData.value !== null)

// watch(paks, () => canRenderTree.value = true);

// 文件变化时更新
watch(pakData, async () => {
  console.debug('pakData changed', pakData.value)
  treeData.value = null
  // sync to work store
  workStore.unpack.paks = pakData.value.map((pak) => pak.path)
})

// auto render tree
watch(
  () => [pakData.value, workStore.unpack.fileList],
  async () => {
    if (workStore.unpack.fileList && pakData.value.length > 0) {
      showOverlay.value = true
      loadingTree.value = false
    }
  }
)

// 更新过滤器
const updateFilter = () => {
  workStore.unpack.filterText = workStore.unpack.filterText.trim()
  filterTextApply.value = workStore.unpack.filterText
}

// function convertNode(fileTreeNode: FileTreeNode): TreeData {
//   const children = Array.from(Object.values(fileTreeNode.children)).map(convertNode);
//   return {
//     label: fileTreeNode.info.relativePath,
//     hash: fileTreeNode.info.hash,
//     isDir: fileTreeNode.info.isDir,
//     children,
//   };
// }

async function handleOpen() {
  try {
    let result = await dialogOpen({
      multiple: true,
      filters: [
        {
          name: 'RE Engine Pak',
          extensions: ['pak']
        }
      ]
    })
    if (!result) {
      console.log('No file selected')
      return
    }
    if (typeof result === 'string') {
      result = [result]
    }

    for (const filePath of result) {
      await pak_open(filePath)
    }
    await reloadData()
  } catch (error) {
    ShowError(error)
  }
}

async function handleClose(index: number) {
  try {
    const pak = pakData.value[index]
    if (!pak) {
      return
    }

    console.log('Closing Pak', pak.path)
    await pak_close(pak.id)

    await reloadData()
  } catch (error) {
    ShowError(error)
  }
}

// 点击 Render 按钮后的事件回调
async function doRender() {
  loadingTree.value = true
  try {
    // 载入文件名列表
    await file_table_load(workStore.unpack.fileList)
    // 渲染树
    const result = await pak_read_file_tree_optimized()
    treeData.value = result
    showOverlay.value = false
  } catch (error) {
    ShowError(error)
  } finally {
    loadingTree.value = false
  }
}

const handleOrder = async () => {
  // reload pak list
  await reloadData()
}

async function handleCloseAll() {
  try {
    console.log('Closing all paks')
    for (const pak of pakData.value) {
      await pak_close(pak.id)
    }
    await reloadData()
  } catch (error) {
    ShowError(error)
  }
}

async function doExtraction() {
  try {
    // 请求解压目录
    let selected = await dialogOpen({
      directory: true,
      multiple: false
    })
    if (!selected) {
      console.log('No directory selected')
      return
    }
    if (Array.isArray(selected)) {
      selected = selected[0]
    }

    const options: ExtractOptions = {
      outputPath: selected as string,
      override: true,
      extractAll: false,
      extractFiles: fileTreeComponent.value?.getCheckedNodes() ?? []
    }
    // console.log('Extract options', options)
    const window = getCurrentWindow()
    const onEvent = new Channel<WorkProgressEvent>()
    onEvent.onmessage = (event) => {
      if (event.event === 'workStart') {
        totalFileCount.value = event.data.fileCount
        finishFileCount.value = 0
        window.setProgressBar({
          status: ProgressBarStatus.Normal,
          progress: 0
        })
      } else if (event.event === 'workFinished') {
        unpackWorking.value = false
        if (finishFileCount.value !== totalFileCount.value) {
          finishFileCount.value = totalFileCount.value
        }
        window.setProgressBar({
          status: ProgressBarStatus.None,
          progress: 0
        })
      } else if (event.event === 'fileDone') {
        finishFileCount.value = event.data.finishCount
        currentFile.value = event.data.path
        window.setProgressBar({
          status: ProgressBarStatus.Normal,
          progress: Math.floor(progressValue.value)
        })
      }
    }

    unpackWorking.value = true
    showProgressPanel.value = true

    await pak_extract_all(options, onEvent)
  } catch (error) {
    ShowError(error)
  }
}

const dropInAddPaks = async (filePaths: string[]) => {
  try {
    for (const filePath of filePaths) {
      await pak_open(filePath)
    }
    await reloadData()
  } catch (error) {
    ShowError(error)
  }
}

function getLoadedPaks(): Promise<PakInfo[]> {
  return pak_list_all()
}

async function reloadData() {
  pakData.value = await getLoadedPaks()
}

let unlisten: UnlistenFn

async function startListenToDrop() {
  unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type === 'drop') {
      await dropInAddPaks(event.payload.paths)
    }
  })
}

async function stopListenToDrop() {
  unlisten?.()
}

function resetProgress() {
  finishFileCount.value = 0
  totalFileCount.value = 0
  currentFile.value = ''
}

async function handleCloseProgress() {
  if (unpackWorking.value) {
    showConfirmTermination.value = true
    return
  }
  resetProgress()
  showProgressPanel.value = false
}

async function handleConfirmTermination() {
  await pak_terminate_extraction()

  unpackWorking.value = false
  showConfirmTermination.value = false
  resetProgress()
  showProgressPanel.value = false
  console.log('Termination confirmed')
  ShowWarn('Extraction terminated.')
}

// 处理文件拖拽功能
// watch(() => enableAddPak, (allowAdd) => {
//   if (allowAdd) {
//     startListenToDrop()
//   } else {
//     stopListenToDrop()
//   }
// })

async function loadWorkRecords() {
  await workStore.loadWorkRecords()
  if (initialLoaded.value) {
    return
  }

  // initial load
  if (pakData.value.length === 0 && workStore.unpack.paks.length > 0) {
    // load paks
    // check if all paks are exist
    const existsList = await Promise.all(
      workStore.unpack.paks.map(async (path) => {
        return await exists(path)
      })
    )
    const allExists = existsList.every((exist) => exist)

    if (allExists) {
      // all paks are exist, load them
      for (const path of workStore.unpack.paks) {
        await pak_open(path)
      }
    }
  }

  initialLoaded.value = true
}

onMounted(async () => {
  await startListenToDrop()
  // 加载工作记录
  try {
    await loadWorkRecords()
  } catch (error) {
    // ignore error
    console.error(error)
  }
  // 加载数据
  await reloadData()
})

onUnmounted(async () => {
  await stopListenToDrop()
})
</script>

<style scoped lang="scss">
.main-container {
  height: 100%;
}

.aside-outer {
  width: 300px;
}

.aside-container {
  display: flex;
  flex-direction: column;
  row-gap: 1rem;
  // margin: 0 10px;
  margin-right: 10px;
}

.block-text {
  display: inline-block;
  align-self: normal;
}

.tool-chunk {
  display: flex;
  flex-flow: column;
  row-gap: 10px;
}

.main-content {
  height: 100%;
  width: 100%;
  padding: 0 0.5rem 1rem 0.5rem;

  .tree-card {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }

  .tree-panel {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
    overflow: auto;
    padding: 10px;

    .file-tree {
      flex-grow: 1;
      overflow: auto;
    }
  }
}

.progress-actions {
  display: flex;
  justify-content: right;
  margin: 8px;
}

.tree-actions {
  margin-top: 20px;
}

.v-card-text {
  .text-body-2 {
    display: block;
    height: 1.5em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(192, 192, 192, 0.5);
  backdrop-filter: blur(2px);
  z-index: 10;
  display: flex;
  justify-content: center;
  align-items: center;
}

.overlay-content {
  padding: 24px;
}

.load-btn {
  min-width: 160px;
}
</style>
