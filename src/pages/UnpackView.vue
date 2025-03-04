<script setup lang="ts">
import { pak_close, pak_extract_all, pak_list_all, pak_open, pak_read_file_tree_optimized } from '@/api/tauri/pak';
import type { ExtractOptions, PakInfo, RenderTreeNode } from '@/api/tauri/pak';
import PakFiles from '@/components/PakFiles.vue'
import FileTree from '@/components/FileTree.vue'
import FileNameTableSelector from '@/components/FileNameTableSelector.vue'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
// import { listen, TauriEvent as TauriEventName, type Event as TauriEvent } from '@tauri-apps/api/event'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import { file_table_load } from '@/api/tauri/filelist';
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { UnlistenFn } from '@tauri-apps/api/event';

// const pakStore = usePakStore();

// 过滤器输入（原始输入）
const filterTextInput = ref('');
// 过滤器输入（已处理）
const filterText = ref('');
// 已加载的pak
const pakData = ref<PakInfo[]>([])
// 树视图加载状态
const loading = ref(false);
// 树视图数据
const treeData = ref<RenderTreeNode | null>(null);
const fileNameTablePath = ref('');
// 是否允许添加Pak文件
const enableAddPaks = computed(() => { return fileNameTablePath.value !== '' })

const fileTreeComponent = ref<InstanceType<typeof FileTree>>();
// const paks = ref<Map<string, PakId>>(new Map());
// const canRenderTree = ref(false);
// // 启用FileList选择
// const enableFileListSelect = computed(() => pakData.value.length > 0);
// 启用树渲染按键
// const enableTreeRender = computed(() => treeData.value ? treeData > 0 : false);
// 启用解压按键
const enableExtract = computed(() => treeData.value !== null)

// watch(paks, () => canRenderTree.value = true);

// 文件变化时清空当前树和过滤器
watch(pakData, () => {
  treeData.value = null;
  filterText.value = '';
})

// 更新过滤器
const updateFilter = () => {
  const input = filterTextInput.value.trim();
  if (input !== filterText.value) {
    filterText.value = filterTextInput.value;
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
  fileNameTablePath.value = filePath;
}

async function handleOpen() {
  try {
    let result = await dialogOpen({
      multiple: true, filters: [{
        name: 'RE Engine Pak',
        extensions: ['pak']
      }]
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
    console.error(error)
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
    console.error(error)
  }
}

// 点击 Render 按钮后的事件回调
async function handleRender() {
  try {
    // 载入文件名列表
    await file_table_load(fileNameTablePath.value)

    // 渲染树
    const result = await pak_read_file_tree_optimized()
    treeData.value = result

  } catch (error) {
    console.error(error)
  }
}

async function doExtract() {
  try {
    // 请求解压目录
    let selected = await dialogOpen({
      directory: true,
      multiple: false,
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
      extractFiles: fileTreeComponent.value?.getCheckedNodes() || [],
    }
    // console.log('Extract options', options)
    await pak_extract_all(options)

  } catch (error) {
    console.error(error)
  } finally {
    loading.value = false
  }
}

async function dropInAddPaks(filePaths: string[]) {
  try {
    for (const filePath of filePaths) {
      await pak_open(filePath)
    }
    await reloadData()
  } catch (error) {
    console.error(error)
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
          <el-text class="block-text">File Name Table</el-text>
          <FileNameTableSelector @change="onFileNameTableChange" :disabled="false">
          </FileNameTableSelector>
        </div>
        <div class="tool-chunk">
          <el-text class="block-text">Pak Files</el-text>
          <PakFiles :pak-list="pakData" :enable-add="enableAddPaks" @open="handleOpen" @close="handleClose"
            @render="handleRender"></PakFiles>
        </div>
        <div class="tool-chunk">
          <el-text class="block-text">Filter</el-text>
          <el-input v-model="filterTextInput" placeholder="Filter keyword" />
          <el-button type="primary" @click="updateFilter">Apply Filter</el-button>
        </div>
      </div>
    </el-aside>
    <el-main v-loading="loading">
      <FileTree ref="fileTreeComponent" :data="treeData" :filter-text="filterText"></FileTree>
      <el-button-group class="tree-actions">
        <el-button type="primary" @click="doExtract" :disabled="!enableExtract">Extract</el-button>
      </el-button-group>
    </el-main>
  </el-container>
</template>

<style scoped>
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
  row-gap: 1rem;
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

.tree-actions {
  margin-top: 20px;

}
</style>