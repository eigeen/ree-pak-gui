<template>
  <section class="desktop-page">
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel :default-size="24" :max-size="42" :min-size="18">
          <aside class="flex h-full min-w-0 flex-col bg-[#121214]">
            <div class="desktop-toolbar">
              <div class="flex items-center gap-1">
                <button
                  type="button"
                  class="desktop-side-tab"
                  :class="sidebarTab === 'resources' ? 'desktop-side-tab-active' : ''"
                  @click="sidebarTab = 'resources'"
                >
                  <PackageOpen class="size-4" />
                  <span>Resources</span>
                </button>
                <button
                  type="button"
                  class="desktop-side-tab"
                  :class="sidebarTab === 'tree' ? 'desktop-side-tab-active' : ''"
                  @click="sidebarTab = 'tree'"
                >
                  <FolderTree class="size-4" />
                  <span>Tree</span>
                </button>
              </div>
            </div>

            <div class="editor-scrollbar flex min-h-0 flex-1 flex-col gap-3 overflow-auto p-3">
              <section v-if="sidebarTab === 'resources'" class="flex min-h-0 flex-1 flex-col">
                <div class="mb-3 flex items-center justify-between gap-3">
                  <div>
                    <p class="section-eyebrow">
                      {{ t('unpack.fileList') }} / {{ t('unpack.pakFiles') }}
                    </p>
                    <h2 class="section-title">Resources</h2>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="desktop-pill max-w-28 truncate">{{
                      unpackState.fileList || 'None'
                    }}</span>
                    <span class="desktop-pill">{{ pakData.length }}</span>
                  </div>
                </div>
                <div class="mb-4">
                  <FileNameTable v-model="unpackState.fileList" :show-manage-button="false" />
                </div>
                <div class="min-h-0 flex-1">
                  <PakFiles
                    :enable-add="enableAddPaks"
                    :pak-list="pakData"
                    :show-open-button="false"
                    @close="handleClose"
                    @close-all="handleCloseAll"
                    @open="handleOpen"
                    @order="handleOrder"
                  />
                </div>
              </section>

              <section v-else class="flex min-h-0 flex-1 flex-col overflow-hidden">
                <div class="mb-3 flex items-center gap-2">
                  <DenseInput
                    v-model="unpackState.filterText"
                    :placeholder="t('unpack.filterKeyword')"
                  />
                  <Button
                    variant="outline"
                    size="sm"
                    class="desktop-command-button"
                    :disabled="unpackState.filterText === filterTextApply"
                    @click="updateFilter"
                  >
                    <Filter class="size-4" />
                  </Button>
                </div>
                <label class="mb-3 flex items-center gap-2 text-[12px] text-muted-foreground">
                  <input v-model="unpackState.filterUseRegex" class="size-4" type="checkbox" />
                  <span>{{ t('unpack.regex') }}</span>
                </label>

                <div
                  class="desktop-toolbar h-8 min-h-8 justify-between border-x-0 border-t border-border/80 px-0"
                >
                  <div class="flex items-center gap-1.5">
                    <Button
                      size="icon-sm"
                      variant="ghost"
                      class="desktop-icon-button"
                      :disabled="!activeTreeNodeKey"
                      title="Bring selected file/folder to view"
                      @click="bringSelectedEntryIntoTreeView"
                    >
                      <LocateFixed class="size-4" />
                    </Button>
                    <Button
                      size="icon-sm"
                      variant="ghost"
                      class="desktop-icon-button"
                      :disabled="!treeData"
                      title="Collapse all"
                      @click="collapseTree"
                    >
                      <FoldVertical class="size-4" />
                    </Button>
                  </div>
                  <Button
                    size="icon-sm"
                    variant="ghost"
                    class="desktop-icon-button"
                    :disabled="!showOverlay"
                    @click="doRender"
                  >
                    <RefreshCw class="size-4" :class="loadingTree ? 'animate-spin' : ''" />
                  </Button>
                </div>

                <div class="min-h-0 flex-1 pt-3">
                  <div v-if="!treeData" class="empty-state h-full border-0 bg-transparent">
                    <FileArchive class="size-8 text-muted-foreground" />
                    <p class="text-sm font-medium text-foreground">
                      {{ pakData.length === 0 ? '等待载入 Pak 文件' : '等待生成文件树' }}
                    </p>
                    <p class="section-copy">
                      {{
                        pakData.length === 0
                          ? '添加 Pak 与路径列表后即可开始浏览。'
                          : '点击刷新按钮载入资源树。'
                      }}
                    </p>
                  </div>

                  <FileTree
                    v-else
                    ref="fileTreeComponent"
                    :current-node-key="activeTreeNodeKey"
                    :data="treeData"
                    :filter-text="filterTextApply"
                    :regex-mode="unpackState.filterUseRegex"
                    class="h-full"
                    @node-click="handleNodeClick"
                  />
                </div>
              </section>
            </div>
          </aside>
        </ResizablePanel>

        <ResizableHandle class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary" />

        <ResizablePanel :default-size="72" :min-size="48">
          <ResizablePanelGroup direction="vertical">
            <ResizablePanel :default-size="75" :min-size="52">
              <div class="flex h-full min-w-0 flex-col">
                <div class="desktop-toolbar justify-between">
                  <div class="flex min-w-0 flex-1 items-center gap-2">
                    <Search class="size-4 text-muted-foreground" />
                    <DenseInput
                      v-model="explorerSearchText"
                      class="w-44 border-border/60 bg-background/80"
                      placeholder="Search current folder..."
                    />
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    class="desktop-command-button"
                    :disabled="!enableExtract"
                    @click="doExtraction"
                  >
                    <Download class="size-4" />
                    {{ t('unpack.extract') }}
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    class="desktop-icon-button"
                    @click="previewPanelEnabled = !previewPanelEnabled"
                  >
                    <PanelRightClose class="size-4" />
                  </Button>
                </div>

                <div class="desktop-subtoolbar">
                  <div
                    v-for="(segment, index) in breadcrumbDisplaySegments"
                    :key="`${segment.id}-${index}-${segment.label}`"
                    class="flex min-w-0 items-center"
                  >
                    <span v-if="index > 0" class="px-1 text-muted-foreground/80">/</span>
                    <button
                      type="button"
                      class="truncate transition-colors hover:text-foreground"
                      :class="segment.id === currentDirectoryKey ? 'font-medium text-foreground' : ''"
                      @click="openDirectory(segment.id)"
                    >
                      {{ segment.label }}
                    </button>
                  </div>
                </div>

                <ResizablePanelGroup direction="horizontal" class="min-h-0 flex-1">
                  <ResizablePanel :default-size="previewPanelEnabled ? 72 : 100" :min-size="48">
                    <div class="relative h-full min-w-0">
                      <div
                        v-if="showOverlay"
                        class="absolute inset-4 z-20 flex items-center justify-center border border-border/80 bg-background/88 backdrop-blur-sm"
                        @click.stop
                      >
                        <Button :disabled="loadingTree" @click="doRender">
                          <RefreshCw class="size-4" :class="loadingTree ? 'animate-spin' : ''" />
                          {{ t('unpack.loadFileTree') }}
                        </Button>
                      </div>

                      <div class="h-full p-4">
                        <div v-if="pakData.length === 0" class="empty-state h-full">
                          <FileArchive class="size-10 text-muted-foreground" />
                          <p class="text-sm font-semibold text-foreground">尚未添加文件</p>
                          <p class="section-copy">点击左侧按钮或拖拽文件到窗口中添加 Pak 文件。</p>
                        </div>

                        <div v-else-if="!treeData" class="empty-state h-full">
                          <FolderTree class="size-10 text-muted-foreground" />
                          <p class="text-sm font-semibold text-foreground">资源树尚未载入</p>
                          <p class="section-copy">
                            选择路径列表后，点击左侧刷新按钮生成 Explorer。
                          </p>
                        </div>

                        <div
                          v-else
                          class="editor-scrollbar grid h-full grid-cols-[repeat(auto-fill,minmax(7.25rem,1fr))] gap-2 overflow-auto pr-1 content-start"
                        >
                          <button
                            v-for="item in explorerEntries"
                            :key="item.id"
                            type="button"
                            class="group flex h-30 flex-col border border-border/80 bg-[#151518] px-2.5 py-2.5 text-left transition-colors hover:border-primary/35 hover:bg-[#1b1b20]"
                            :class="
                              selectedEntryKey === item.id
                                ? 'border-primary bg-primary/12 shadow-[inset_0_0_0_1px_rgba(116,169,255,0.22)]'
                                : ''
                            "
                            @click="handleExplorerItemClick(item)"
                            @dblclick="handleExplorerItemOpen(item)"
                          >
                            <div
                              class="mb-2 flex size-12 items-center justify-center border border-border/80 bg-secondary/75"
                              :class="item.isDir ? 'text-amber-200' : 'text-primary'"
                            >
                              <Folder v-if="item.isDir" class="size-7" />
                              <File v-else class="size-6" />
                            </div>
                            <p class="truncate text-[13px] font-semibold leading-5 text-foreground">
                              {{ item.name }}
                            </p>
                            <p class="mt-0.5 text-[10px] text-muted-foreground">
                              {{ item.isDir ? `${item.children.length} items` : item.sizeText }}
                            </p>
                            <p
                              class="mt-auto truncate pt-2 text-[9px] uppercase tracking-[0.18em] text-muted-foreground"
                            >
                              {{ item.isDir ? 'Folder' : 'Asset' }}
                            </p>
                          </button>
                        </div>
                      </div>
                    </div>
                  </ResizablePanel>

                  <template v-if="previewPanelEnabled">
                    <ResizableHandle
                      class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary"
                    />
                    <ResizablePanel :default-size="28" :max-size="40" :min-size="20">
                      <div class="h-full border-l border-border/80 bg-[#101012] p-3">
                        <PreviewPane :file-name="previewFileName" :preview-uri="previewUri" />
                      </div>
                    </ResizablePanel>
                  </template>
                </ResizablePanelGroup>
              </div>
            </ResizablePanel>

            <ResizableHandle class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary" />

            <ResizablePanel :default-size="25" :max-size="42" :min-size="16">
              <div class="flex h-full min-w-0 flex-col bg-[#09090b]">
                <div class="flex min-h-0 flex-1">
                  <div class="w-[18rem] shrink-0 border-r border-border/80 p-4">
                    <div
                      class="mb-3 flex items-center gap-2 text-[10px] font-semibold uppercase tracking-[0.22em] text-muted-foreground"
                    >
                      <span>Information</span>
                      <div class="h-px flex-1 bg-border/80" />
                    </div>
                    <div class="space-y-2 text-xs">
                      <div class="flex items-center justify-between gap-3">
                        <span class="text-muted-foreground">Packages Count</span>
                        <span>{{ pakData.length }}</span>
                      </div>
                      <div class="flex items-center justify-between gap-3">
                        <span class="text-muted-foreground">Folders Count</span>
                        <span>{{ currentDirectoryStats.folderCount }}</span>
                      </div>
                      <div class="flex items-center justify-between gap-3">
                        <span class="text-muted-foreground">Assets Count</span>
                        <span>{{ currentDirectoryStats.fileCount }}</span>
                      </div>
                      <div class="flex items-center justify-between gap-3">
                        <span class="text-muted-foreground">Selected</span>
                        <span class="truncate">{{
                          selectedEntry?.name ?? currentDirectory?.name ?? 'Root'
                        }}</span>
                      </div>
                      <div class="flex items-center justify-between gap-3">
                        <span class="text-muted-foreground">Preview</span>
                        <span>{{ previewPanelEnabled ? 'Docked' : 'Hidden' }}</span>
                      </div>
                    </div>
                  </div>

                  <div
                    class="editor-scrollbar min-w-0 flex-1 overflow-auto p-3 font-mono text-[11px] leading-5"
                  >
                    <div
                      v-for="(line, index) in consoleLines"
                      :key="`${index}-${line}`"
                      :class="
                        line.startsWith('[ERR]')
                          ? 'text-destructive'
                          : line.startsWith('[WRN]')
                            ? 'text-amber-400'
                            : index === consoleLines.length - 1
                              ? 'text-primary'
                              : 'text-muted-foreground'
                      "
                    >
                      {{ line }}
                    </div>
                  </div>
                </div>
              </div>
            </ResizablePanel>
          </ResizablePanelGroup>
        </ResizablePanel>
      </ResizablePanelGroup>

      <div class="desktop-statusbar">
        <div class="flex items-center gap-3">
          <span>{{ statusText }}</span>
          <span v-if="loadingTree">Loading tree...</span>
        </div>
        <div class="flex items-center gap-4">
          <span>{{ currentDirectoryPath }}</span>
          <span>Last Refresh: {{ lastRefreshText }}</span>
        </div>
      </div>
    </div>

    <Dialog v-model:open="showProgressPanel">
      <DialogContent
        class="max-w-lg rounded-[1rem] border-border/80 bg-background/96"
        :show-close-button="false"
      >
        <DialogHeader>
          <DialogTitle>{{ t('unpack.extractingFiles') }}</DialogTitle>
          <DialogDescription>
            <span v-if="!unpackWorking">{{ t('unpack.done') }}</span>
            <span v-else>处理中，请稍候。</span>
          </DialogDescription>
        </DialogHeader>

        <div class="space-y-4">
          <Progress :model-value="progressValue" class="h-2 rounded-full" />
          <p class="text-sm text-muted-foreground">
            {{ finishFileCount }} / {{ totalFileCount }} {{ t('unpack.files') }}
          </p>
          <div class="space-y-1">
            <p class="text-sm font-medium text-foreground">{{ t('unpack.extracting') }}</p>
            <p class="truncate text-sm text-muted-foreground">{{ currentFile }}</p>
          </div>
        </div>

        <DialogFooter>
          <Button :variant="unpackWorking ? 'destructive' : 'outline'" @click="handleCloseProgress">
            {{ unpackWorking ? t('unpack.terminate') : t('unpack.close') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <AlertDialog v-model:open="showConfirmTermination">
      <AlertDialogContent class="rounded-[1rem] border-border/80 bg-background/96">
        <AlertDialogHeader>
          <AlertDialogTitle>{{ t('unpack.confirmTermination') }}</AlertDialogTitle>
          <AlertDialogDescription>{{ t('unpack.confirmTerminationText') }}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{{ t('unpack.cancel') }}</AlertDialogCancel>
          <AlertDialogAction @click="handleConfirmTermination">
            {{ t('unpack.confirm') }}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { Channel, convertFileSrc } from '@tauri-apps/api/core'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { ProgressBarStatus, getCurrentWindow } from '@tauri-apps/api/window'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import { exists } from '@tauri-apps/plugin-fs'
import {
  Download,
  File,
  FileArchive,
  FoldVertical,
  Filter,
  Folder,
  FolderTree,
  LocateFixed,
  PackageOpen,
  PanelRightClose,
  RefreshCw,
  Search
} from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import {
  pak_close,
  pak_extract_all,
  pak_list_all,
  pak_open,
  pak_read_file_tree_optimized,
  pak_terminate_extraction
} from '@/api/tauri/pak'
import type {
  ExtractOptions,
  JsSafeHash,
  PakInfo,
  RenderTreeNode,
  UnpackProgressEvent
} from '@/api/tauri/pak'
import { getPreviewFile } from '@/api/tauri/utils'
import FileTree, { type TreeData } from '@/components/FileTree.vue'
import FileNameTable from '@/components/FileNameTable/FileNameTable.vue'
import PakFiles from '@/components/PakFiles.vue'
import PreviewPane from '@/components/PreviewPane.vue'
import { fileListService } from '@/service/filelist'
import { useWorkStore } from '@/store/work'
import { ShowError, ShowWarn } from '@/utils/message'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog'
import { DenseInput } from '@/components/ui/input'
import { Progress } from '@/components/ui/progress'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'

type UnpackState = {
  fileList: string
  paks: string[]
  filterText: string
  filterUseRegex: boolean
}

type ExplorerEntry = TreeData & {
  children: ExplorerEntry[]
}

type SidebarTab = 'resources' | 'tree'

const { t } = useI18n()
const workStore = useWorkStore()

const unpackState = computed({
  get: () => workStore.unpack as unknown as UnpackState,
  set: (value: UnpackState) => {
    ;(workStore as any).unpack = value
  }
})

const filterTextApply = ref('')
const explorerSearchText = ref('')
const sidebarTab = ref<SidebarTab>('resources')
const pakData = ref<PakInfo[]>([])
const initialLoaded = ref(false)
const treeData = ref<RenderTreeNode | null>(null)
const showOverlay = ref(false)
const loadingTree = ref(false)
const previewPanelEnabled = ref(false)
const previewUri = ref('')
const previewFileName = ref('')
const currentDirectoryKey = ref('')
const selectedEntryKey = ref('')
const unpackWorking = ref(false)
const showProgressPanel = ref(false)
const currentFile = ref('')
const totalFileCount = ref(0)
const finishFileCount = ref(0)
const showConfirmTermination = ref(false)
const lastRefreshAt = ref<Date | null>(null)
const consoleLines = ref<string[]>([
  '[INF] Workbench ready.',
  '[INF] Waiting for pak files and a path list.'
])

const progressValue = computed(() =>
  totalFileCount.value === 0 ? 0 : (finishFileCount.value / totalFileCount.value) * 100
)

const enableAddPaks = computed(() => unpackState.value.fileList !== '')
const enableExtract = computed(() => treeData.value !== null)
const fileTreeComponent = ref<InstanceType<typeof FileTree>>()

const explorerRoot = computed<ExplorerEntry | null>(() =>
  treeData.value ? buildExplorerTree(treeData.value) : null
)

const explorerNodeMap = computed(() => {
  const map = new Map<string, ExplorerEntry>()

  const walk = (node: ExplorerEntry | null) => {
    if (!node) return
    map.set(node.id, node)
    node.children.forEach(walk)
  }

  walk(explorerRoot.value)
  return map
})

const currentDirectory = computed(() => {
  const node = currentDirectoryKey.value
    ? explorerNodeMap.value.get(currentDirectoryKey.value)
    : undefined
  return node?.isDir ? node : (explorerRoot.value ?? null)
})

const selectedEntry = computed(() =>
  selectedEntryKey.value ? explorerNodeMap.value.get(selectedEntryKey.value) : undefined
)

const explorerEntries = computed(() => {
  const dir = currentDirectory.value
  if (!dir) return []

  const keyword = explorerSearchText.value.trim().toLowerCase()
  return dir.children
    .filter((item) => {
      if (!keyword) return true
      return item.name.toLowerCase().includes(keyword) || item.path.toLowerCase().includes(keyword)
    })
    .sort((a, b) => {
      if (a.isDir !== b.isDir) return a.isDir ? -1 : 1
      return a.name.localeCompare(b.name)
    })
})

const currentDirectoryStats = computed(() => {
  const dir = currentDirectory.value
  if (!dir) return { folderCount: 0, fileCount: 0 }

  return {
    folderCount: dir.children.filter((item) => item.isDir).length,
    fileCount: dir.children.filter((item) => !item.isDir).length
  }
})

const activeTreeNodeKey = computed(() => selectedEntry.value?.id ?? currentDirectoryKey.value)
const currentDirectoryPath = computed(() => currentDirectory.value?.path ?? 'Root')
const statusText = computed(() => {
  if (unpackWorking.value) return 'Extracting'
  if (loadingTree.value) return 'Loading tree'
  if (!treeData.value) return 'Idle'
  return 'Completed'
})
const lastRefreshText = computed(() =>
  lastRefreshAt.value ? lastRefreshAt.value.toLocaleTimeString() : 'Never'
)

const breadcrumbDisplaySegments = computed(() => {
  const segments: Array<{ id: string; label: string }> = []
  let cursor = currentDirectory.value

  while (cursor) {
    const labels = splitBreadcrumbLabel(cursor.name)
    for (let i = labels.length - 1; i >= 0; i -= 1) {
      segments.unshift({ id: cursor.id, label: labels[i] ?? cursor.name })
    }
    cursor = cursor.parentId ? (explorerNodeMap.value.get(cursor.parentId) ?? null) : null
  }

  return segments
})

function splitBreadcrumbLabel(label: string): string[] {
  if (label === '/') {
    return ['/']
  }

  const parts = label.split(/\s*\/\s*/).filter((part) => part.length > 0)
  return parts.length > 0 ? parts : [label]
}

watch(pakData, async () => {
  treeData.value = null
  currentDirectoryKey.value = ''
  selectedEntryKey.value = ''
  previewUri.value = ''
  previewFileName.value = ''
  if (initialLoaded.value) {
    unpackState.value.paks = pakData.value.map((pak) => pak.path)
  }
})

watch(
  () => [pakData.value, unpackState.value.fileList],
  async () => {
    if (unpackState.value.fileList && pakData.value.length > 0) {
      showOverlay.value = true
      loadingTree.value = false
    }
  }
)

watch(explorerRoot, (root) => {
  if (!root) {
    currentDirectoryKey.value = ''
    return
  }

  currentDirectoryKey.value = explorerNodeMap.value.has(currentDirectoryKey.value)
    ? currentDirectoryKey.value
    : root.id
})

watch(selectedEntryKey, async (key) => {
  const entry = key ? explorerNodeMap.value.get(key) : undefined
  if (!entry || entry.isDir) {
    previewUri.value = ''
    previewFileName.value = ''
    return
  }

  try {
    const previewFile = await getPreviewFile(entry.hash ?? parseId(entry.id))
    previewUri.value = convertFileSrc(previewFile, 'asset')
    previewFileName.value = entry.name
    previewPanelEnabled.value = true
  } catch {
    previewUri.value = ''
    previewFileName.value = entry.name
  }
})

watch(
  () => enableAddPaks.value,
  async (allowAdd) => {
    if (allowAdd) {
      await startListenToDrop()
    } else {
      await stopListenToDrop()
    }
  }
)

const updateFilter = () => {
  unpackState.value.filterText = unpackState.value.filterText.trim()
  filterTextApply.value = unpackState.value.filterText
  pushConsole(`[INF] Applied filter: ${filterTextApply.value || 'none'}`)
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

    if (!result) return
    if (typeof result === 'string') result = [result]

    for (const filePath of result) {
      await pak_open(filePath)
      pushConsole(`[INF] Opened pak: ${filePath}`)
    }

    await reloadData()
  } catch (error) {
    ShowError(t('global.failedLoadSettings', { error: String(error) }))
    ShowWarn(t('global.useDefaultSettings'))
  }
}

async function handleClose(index: number) {
  try {
    const pak = pakData.value[index]
    if (!pak) return

    await pak_close(pak.id)
    pushConsole(`[INF] Closed pak: ${pak.path}`)
    await reloadData()
  } catch (error) {
    ShowError(error)
  }
}

async function doRender() {
  loadingTree.value = true
  try {
    const file = fileListService.getFileByIdent(unpackState.value.fileList)
    if (!file) {
      throw new Error(`Name list file not found: ${unpackState.value.fileList}`)
    }

    await fileListService.loadFilePathList(file.source.filePath)
    treeData.value = await pak_read_file_tree_optimized()
    showOverlay.value = false
    lastRefreshAt.value = new Date()
    pushConsole('[INF] Explorer tree loaded.')
  } catch (error) {
    ShowError(error)
  } finally {
    loadingTree.value = false
  }
}

const handleOrder = async () => {
  pushConsole('[INF] Reordered pak priority.')
  await reloadData()
}

async function handleCloseAll() {
  try {
    for (const pak of pakData.value) {
      await pak_close(pak.id)
    }
    pushConsole('[WRN] Closed all pak files.')
    await reloadData()
  } catch (error) {
    ShowError(error)
  }
}

async function doExtraction() {
  try {
    let selected = await dialogOpen({
      directory: true,
      multiple: false
    })

    if (!selected) return
    if (Array.isArray(selected)) selected = selected[0]

    const options: ExtractOptions = {
      outputPath: selected as string,
      override: true,
      extractAll: false,
      extractFiles: fileTreeComponent.value?.getCheckedNodes() ?? []
    }

    const window = getCurrentWindow()
    const onEvent = new Channel<UnpackProgressEvent>()

    onEvent.onmessage = (event) => {
      if (event.event === 'workStart') {
        totalFileCount.value = event.data.count
        finishFileCount.value = 0
        pushConsole(`[INF] Extraction started for ${event.data.count} files.`)
        window.setProgressBar({ status: ProgressBarStatus.Normal, progress: 0 })
      } else if (event.event === 'workFinished') {
        unpackWorking.value = false
        if (finishFileCount.value !== totalFileCount.value) {
          finishFileCount.value = totalFileCount.value
        }
        pushConsole('[INF] Extraction finished.')
        window.setProgressBar({ status: ProgressBarStatus.None, progress: 0 })
      } else if (event.event === 'fileDone') {
        finishFileCount.value = event.data.finishCount
        currentFile.value = event.data.path
        pushConsole(`[INF] Exported ${event.data.path}`)
        window.setProgressBar({
          status: ProgressBarStatus.Normal,
          progress: Math.floor(progressValue.value)
        })
      } else if (event.event === 'error') {
        pushConsole(`[ERR] ${event.data.error}`)
      }
    }

    unpackWorking.value = true
    showProgressPanel.value = true
    await pak_extract_all(options, onEvent)
  } catch (error) {
    ShowError(error)
  }
}

async function dropInAddPaks(filePaths: string[]) {
  try {
    for (const filePath of filePaths) {
      await pak_open(filePath)
      pushConsole(`[INF] Dropped pak: ${filePath}`)
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

let unlisten: UnlistenFn | undefined
const handleWindowOpenPaks = () => {
  void handleOpen()
}
const handleWindowRenderTree = () => {
  if (!unpackState.value.fileList || pakData.value.length === 0) return
  void doRender()
}

async function startListenToDrop() {
  if (unlisten) return

  unlisten = await getCurrentWebview().onDragDropEvent(async (event: any) => {
    if (event.payload.type === 'drop') {
      await dropInAddPaks(event.payload.paths)
    }
  })
}

async function stopListenToDrop() {
  await unlisten?.()
  unlisten = undefined
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
  pushConsole('[WRN] Extraction terminated by user.')
  ShowWarn(t('global.extractionTerminated'))
}

function parseId(id: string): JsSafeHash {
  return id.split(',').map((str) => parseInt(str, 10)) as JsSafeHash
}

function handleNodeClick(data: TreeData) {
  if (data.isDir) {
    openDirectory(data.id)
    selectedEntryKey.value = ''
    return
  }

  selectedEntryKey.value = data.id
  currentDirectoryKey.value = data.parentId ?? currentDirectoryKey.value
}

function bringSelectedEntryIntoTreeView() {
  const key = activeTreeNodeKey.value
  if (!key) return

  fileTreeComponent.value?.bringNodeIntoView(key)
}

async function loadWorkRecords() {
  await workStore.loadWorkRecords()
  if (initialLoaded.value) return

  if (pakData.value.length === 0 && unpackState.value.paks.length > 0) {
    const existsList = await Promise.all(
      unpackState.value.paks.map(async (path: string) => exists(path))
    )
    const allExists = existsList.every(Boolean)

    if (allExists) {
      for (const path of unpackState.value.paks) {
        await pak_open(path)
      }
    }
  }

  initialLoaded.value = true
  await reloadData()
  unpackState.value.paks = pakData.value.map((pak) => pak.path)
}

function buildExplorerTree(
  node: RenderTreeNode,
  parentPath = '',
  parentId?: string
): ExplorerEntry {
  const id = node.hash ? node.hash.toString() : `${parentPath}/${node.name}`
  const path = parentPath ? `${parentPath}/${node.name}` : node.name

  return {
    id,
    name: node.name,
    label: node.name,
    path,
    parentId,
    hash: node.hash,
    isDir: node.isDir,
    sizeText: formatSize(
      node.uncompressedSize !== undefined
        ? node.isCompressed
          ? node.uncompressedSize
          : node.compressedSize
        : 0
    ),
    children: node.children?.map((child) => buildExplorerTree(child, path, id)) ?? [],
    belongsTo: node.belongsTo
  }
}

function formatSize(size: number): string {
  if (size < 0) return 'Invalid'

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let index = 0
  let current = size

  while (current >= 1024 && index < units.length - 1) {
    current /= 1024
    index++
  }

  return `${current.toFixed(2)} ${units[index]}`
}

function openDirectory(id: string) {
  const entry = explorerNodeMap.value.get(id)
  if (!entry || !entry.isDir) return
  currentDirectoryKey.value = entry.id
  selectedEntryKey.value = ''
}

function collapseTree() {
  fileTreeComponent.value?.collapseAll()
}

function handleExplorerItemClick(item: ExplorerEntry) {
  if (item.isDir) {
    currentDirectoryKey.value = item.id
    selectedEntryKey.value = ''
    return
  }

  selectedEntryKey.value = item.id
}

function handleExplorerItemOpen(item: ExplorerEntry) {
  if (item.isDir) {
    openDirectory(item.id)
    return
  }

  selectedEntryKey.value = item.id
  previewPanelEnabled.value = true
}

function pushConsole(line: string) {
  consoleLines.value = [...consoleLines.value.slice(-13), line]
}

onMounted(async () => {
  window.addEventListener('unpack:open-paks', handleWindowOpenPaks)
  window.addEventListener('unpack:render-tree', handleWindowRenderTree)
  await startListenToDrop()
  await loadWorkRecords()
})

onUnmounted(async () => {
  window.removeEventListener('unpack:open-paks', handleWindowOpenPaks)
  window.removeEventListener('unpack:render-tree', handleWindowRenderTree)
  await stopListenToDrop()
})
</script>
