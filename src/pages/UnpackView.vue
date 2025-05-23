<script setup lang="ts">
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
import FileNameTableSelector from '@/components/FileNameTableSelector.vue'
import { computed, getCurrentInstance, onMounted, onUnmounted, ref, watch } from 'vue'
// import { listen, TauriEvent as TauriEventName, type Event as TauriEvent } from '@tauri-apps/api/event'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import { file_table_load } from '@/api/tauri/filelist'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { Channel } from '@tauri-apps/api/core'
import { ShowError, ShowWarn } from '@/utils'

// 过滤器输入（原始输入）
const filterTextInput = ref('')
// 过滤器输入（已处理）
const filterText = ref('')
// 已加载的pak
const pakData = ref<PakInfo[]>([])
// 树视图加载状态
const loading = ref(false)
// 树视图数据
const treeData = ref<RenderTreeNode | null>(null)
const fileNameTablePath = ref('')
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
  return fileNameTablePath.value !== ''
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
  filterText.value = ''
})

// auto render tree
watch(
  () => [pakData.value, fileNameTablePath.value],
  async () => {
    if (fileNameTablePath.value && pakData.value.length > 0) {
      await doRender()
    }
  }
)

// 更新过滤器
const updateFilter = () => {
  const input = filterTextInput.value.trim()
  if (input !== filterText.value) {
    filterText.value = filterTextInput.value
  }
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

function onFileNameTableChange(filePath: string) {
  fileNameTablePath.value = filePath
}

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
  try {
    // 载入文件名列表
    await file_table_load(fileNameTablePath.value)

    // 渲染树
    const result = await pak_read_file_tree_optimized()
    treeData.value = result
  } catch (error) {
    ShowError(error)
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

    loading.value = true

    const options: ExtractOptions = {
      outputPath: selected as string,
      override: true,
      extractAll: false,
      extractFiles: fileTreeComponent.value?.getCheckedNodes() || []
    }
    // console.log('Extract options', options)
    const onEvent = new Channel<WorkProgressEvent>()
    onEvent.onmessage = (event) => {
      if (event.event === 'workStart') {
        totalFileCount.value = event.data.fileCount
        finishFileCount.value = 0
      } else if (event.event === 'workFinished') {
        unpackWorking.value = false
        if (finishFileCount.value !== totalFileCount.value) {
          finishFileCount.value = totalFileCount.value
        }
      } else if (event.event === 'fileDone') {
        finishFileCount.value = event.data.finishCount
        currentFile.value = event.data.path
      }
    }

    unpackWorking.value = true
    showProgressPanel.value = true

    await pak_extract_all(options, onEvent)
  } catch (error) {
    ShowError(error)
  } finally {
    loading.value = false
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

async function startListenForDrop() {
  unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type === 'drop') {
      await dropInAddPaks(event.payload.paths)
    }
  })
}

async function stopListenForDrop() {
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
//     startListenForDrop()
//   } else {
//     stopListenForDrop()
//   }
// })

onMounted(async () => {
  await startListenForDrop()
  // 加载数据
  await reloadData()
})

onUnmounted(async () => {
  await stopListenForDrop()
})
</script>

<template>
  <el-container class="main-container">
    <el-aside class="aside-outer">
      <div class="aside-container">
        <v-card class="pa-4 elevation-3 rounded-lg tool-chunk">
          <FileNameTableSelector @change="onFileNameTableChange" :disabled="false">
          </FileNameTableSelector>
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
            v-model="filterTextInput"
            variant="outlined"
            density="comfortable"
            hide-details
            label="Filter keyword"
          ></v-text-field>
          <v-btn class="text-none" prepend-icon="mdi-filter-variant" @click="updateFilter"
            >Apply Filter</v-btn
          >
        </v-card>
      </div>
    </el-aside>

    <div class="main-content">
      <v-card class="pa-2 elevation-3 rounded-lg tree-card">
        <div class="tree-panel">
          <FileTree
            class="file-tree"
            ref="fileTreeComponent"
            :data="treeData"
            :filter-text="filterText"
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
</style>
