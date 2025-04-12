<script setup lang="ts">
import {
  pak_close,
  pak_extract_all,
  pak_list_all,
  pak_open,
  pak_read_file_tree_optimized
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
import { ShowError } from '@/utils'

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
// 进度条
const showProgressPanel = ref(false)
const progressValue = ref(0)
const currentFile = ref('test.bin')
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

async function doExtract() {
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
      // TODO: 进度条更新
      if (event.event === 'start') {
        console.log('Extract start', event.data)
      } else if (event.event === 'progress') {
        console.log('Extract progress', event.data)
      } else if (event.event === 'finished') {
        console.log('Extract end')
      }
    }

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

// // 处理文件拖拽功能
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
        <div class="tool-chunk">
          <FileNameTableSelector @change="onFileNameTableChange" :disabled="false">
          </FileNameTableSelector>
        </div>
        <div class="test-buttons">
          <v-btn
            class="text-none"
            density="compact"
            @click="showProgressPanel = !showProgressPanel"
          >
            Toggle Panel
          </v-btn>
          <v-btn
            class="text-none"
            density="compact"
            @click="progressValue = Math.min(100, progressValue + 10)"
          >
            +10% Progress
          </v-btn>
          <v-btn
            class="text-none"
            density="compact"
            @click="progressValue = Math.max(0, progressValue - 10)"
          >
            -10% Progress
          </v-btn>
          <v-btn
            class="text-none"
            density="compact"
            @click="currentFile = 'test_file_' + Math.floor(Math.random() * 100) + '.dat'"
          >
            Random File
          </v-btn>
        </div>
        <div class="tool-chunk">
          <el-text class="block-text">Pak Files</el-text>
          <PakFiles
            :pak-list="pakData"
            :enable-add="enableAddPaks"
            @open="handleOpen"
            @close="handleClose"
            @order="handleOrder"
            @close-all="handleCloseAll"
          ></PakFiles>
        </div>
        <div class="tool-chunk">
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
        </div>
      </div>
    </el-aside>
    <div v-loading="loading" class="main-content">
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
            color="#409eff"
            prepend-icon="mdi-export"
            @click="doExtract"
            :disabled="!enableExtract"
            >Extract</v-btn
          >
        </div>
      </div>
      <div class="progress-panel" v-show="showProgressPanel">
        <div class="progress-text">Extracting files...</div>
        <v-progress-linear
          :color="progressValue >= 100 ? 'green' : 'primary'"
          height="8"
          :model-value="progressValue"
          rounded
        ></v-progress-linear>
        <div class="progress-detail">Processing: {{ currentFile }} ({{ progressValue }}%)</div>
      </div>
    </div>
  </el-container>
</template>

<style scoped lang="scss">
.main-container {
  height: 100%;
}

.aside-outer {
  width: 240px;
  padding: 1rem 0;
  border-right: 1px solid var(--el-border-color);
}

.aside-container {
  display: flex;
  flex-direction: column;
  row-gap: 1.5rem;
  margin: 0 10px;
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
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;

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

.tree-actions {
  margin-top: 20px;
}

.progress-panel {
  padding: 10px 10px 15px 10px;
  background-color: white;
  border-top: 1px solid #eee;
}

.progress-text {
  margin-bottom: 5px;
  font-weight: bold;
}

.progress-detail {
  margin-top: 5px;
  font-size: 0.8em;
  color: #666;
}

.test-buttons {
  display: flex;
  flex-flow: column;
  gap: 10px;
  padding: 10px;
  background-color: #f5f5f5;
  border-top: 1px solid #ddd;
}
</style>
