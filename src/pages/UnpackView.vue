<template>
  <section class="desktop-page">
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      <PageToolbar :items="desktopMenuItems" />

      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel :default-size="24" :max-size="42" :min-size="18">
          <aside class="surface-sidebar flex h-full min-w-0 flex-col">
            <UnpackSidebarTabs v-model="sidebarTab" :tabs="sidebarTabs" />

            <div class="editor-scrollbar flex min-h-0 flex-1 flex-col gap-3 overflow-auto p-3">
              <section v-show="sidebarTab === 'resources'" class="flex min-h-0 flex-1 flex-col">
                <div class="mb-3 flex items-center justify-between gap-3">
                  <div>
                    <p class="section-eyebrow">
                      {{ t('unpack.fileList') }} / {{ t('unpack.pakFiles') }}
                    </p>
                    <h2 class="section-title">资源</h2>
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

              <section v-show="sidebarTab === 'tree'" class="flex min-h-0 flex-1 flex-col overflow-hidden">
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
                <label class="text-ui-xs mb-3 flex items-center gap-2 text-muted-foreground">
                  <Switch v-model="unpackState.filterUseRegex" />
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
                      :disabled="!bringTargetKey"
                      title="Bring selected file/folder to tree"
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
                    :current-node-key="treeFocusKey"
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
              <UnpackExplorerPane
                v-model:search-text="explorerSearchText"
                :enable-extract="enableExtract"
                :has-tree="Boolean(treeData)"
                :has-pak-data="pakData.length > 0"
                :show-overlay="showOverlay"
                :loading-tree="loadingTree"
                :layout-mode="explorerLayoutMode"
                :items="explorerEntries"
                :selected-key="selectedEntryKey"
                :reset-key="explorerViewResetKey"
                :breadcrumb-segments="breadcrumbDisplaySegments"
                :current-directory-key="currentDirectoryKey"
                :can-go-parent-directory="Boolean(currentDirectory?.parentId)"
                :texture-preview-enabled="texturePreviewEnabled"
                :renderers="explorerRenderers"
                :column-labels="explorerColumnLabels"
                @extract="doExtraction"
                @render="doRender"
                @open-directory="openDirectory"
                @open-parent-directory="openParentDirectory"
                @toggle-layout="toggleExplorerLayout"
                @item-click="handleExplorerItemClick"
                @item-open="handleExplorerItemOpen"
                @visible-items-change="handleVisibleExplorerItemsChange"
              />
            </ResizablePanel>

            <ResizableHandle class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary" />

            <ResizablePanel :default-size="25" :max-size="42" :min-size="16">
              <div class="surface-console flex h-full min-w-0 flex-col">
                <div
                  ref="consoleContainer"
                  class="surface-console-panel text-ui-2xs editor-scrollbar min-h-0 min-w-0 flex-1 overflow-auto border border-border/60 px-3 py-2 font-mono"
                >
                  <div
                    v-for="line in consoleLines"
                    :key="line.id"
                    class="min-w-0 whitespace-pre-wrap break-all"
                  >
                    <span class="text-muted-foreground/70"
                      >[{{ formatLogTime(line.createdAt) }}]</span
                    >
                    <span class="mx-1 font-semibold" :class="getLogLevelClass(line.level)">
                      {{ getLogLevelLabel(line.level) }}
                    </span>
                    <span :class="getLogMessageClass(line.level)">{{ line.message }}</span>
                  </div>

                  <div
                    v-if="consoleLines.length === 0"
                    class="flex h-full items-center text-muted-foreground"
                  >
                    暂无 system 日志
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

    <el-image-viewer
      v-if="imageViewerState.open"
      :url-list="imageViewerState.urls"
      :initial-index="imageViewerState.index"
      :hide-on-click-modal="true"
      :close-on-press-escape="true"
      :teleported="true"
      @close="closeImageViewer"
    />

    <FileNameTable
      ref="fileNameTable"
      v-model="unpackState.fileList"
      :show-manage-button="false"
      :show-selector="false"
      class="hidden"
    />
  </section>
</template>

<script setup lang="ts">
import {
  computed,
  nextTick,
  onMounted,
  onUnmounted,
  ref,
  shallowRef,
  toRef,
  watch,
  type CSSProperties
} from 'vue'
import { Channel, convertFileSrc } from '@tauri-apps/api/core'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { ProgressBarStatus, getCurrentWindow } from '@tauri-apps/api/window'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import { exists } from '@tauri-apps/plugin-fs'
import {
  FoldVertical,
  Filter,
  FolderOpen,
  Wrench,
  LocateFixed,
  PackageOpen,
  RefreshCw,
  FolderTree,
  FileArchive
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
import type { ExtractOptions, PakInfo, RenderTreeNode, UnpackProgressEvent } from '@/api/tauri/pak'
import { getPreviewFile } from '@/api/tauri/utils'
import FileTree, { type TreeData } from '@/components/FileTree.vue'
import type { MenuGroup } from '@/components/DesktopMenuBar.vue'
import FileNameTable from '@/components/FileNameTable/FileNameTable.vue'
import PageToolbar from '@/components/PageToolbar.vue'
import PakFiles from '@/components/PakFiles.vue'
import UnpackSidebarTabs, {
  type UnpackSidebarTabItem
} from '@/components/unpack/UnpackSidebarTabs.vue'
import UnpackExplorerPane from '@/components/unpack/UnpackExplorerPane.vue'
import {
  getExplorerFileTypeDefinition,
  getExplorerThemeForType,
  resolveExplorerFileTypeKey
} from '@/lib/explorerTypeTheme'
import type {
  ExplorerColumnLabels,
  ExplorerDirectoryCounts,
  ExplorerEntry,
  ExplorerLayoutMode,
  ExplorerRenderers
} from '@/lib/unpackExplorer'
import { fileListService } from '@/service/filelist'
import { useSettingsStore } from '@/store/settings'
import { useSystemLogStore, type SystemLogEntry, type SystemLogLevel } from '@/store/system'
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
import { Progress } from '@/components/ui/progress'
import { Switch } from '@/components/ui/switch'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'

type UnpackState = {
  fileList: string
  paks: string[]
  filterText: string
  filterUseRegex: boolean
  explorerLayoutMode: ExplorerLayoutMode
}

type SidebarTab = 'resources' | 'tree'

const { t } = useI18n()
const workStore = useWorkStore()
const settingsStore = useSettingsStore()
const systemLogStore = useSystemLogStore()
const systemLogEntries = toRef(systemLogStore, 'entries')
const isProductionBuild = import.meta.env.PROD

const unpackState = computed({
  get: () => workStore.unpack as unknown as UnpackState,
  set: (value: UnpackState) => {
    ;(workStore as any).unpack = value
  }
})

const filterTextApply = ref('')
const explorerSearchText = ref('')
const sidebarTab = ref<SidebarTab>('resources')
const sidebarTabs: UnpackSidebarTabItem[] = [
  {
    value: 'resources',
    label: 'Resources',
    icon: PackageOpen
  },
  {
    value: 'tree',
    label: 'Tree',
    icon: FolderTree
  }
]
const pakData = ref<PakInfo[]>([])
const initialLoaded = ref(false)
const treeData = ref<RenderTreeNode | null>(null)
const showOverlay = ref(false)
const loadingTree = ref(false)
const currentDirectoryKey = ref('')
const treeFocusKey = ref('')
const selectedEntryKey = ref('')
const unpackWorking = ref(false)
const showProgressPanel = ref(false)
const currentFile = ref('')
const totalFileCount = ref(0)
const finishFileCount = ref(0)
const showConfirmTermination = ref(false)
const consoleLines = shallowRef<SystemLogEntry[]>([])
const consoleContainer = ref<HTMLElement | null>(null)
const visibleExplorerEntries = ref<ExplorerEntry[]>([])
const texturePreviewCache = ref<Record<string, string | null>>({})
const texturePreviewPending = new Set<string>()
const explorerLayoutMode = ref<ExplorerLayoutMode>('details')
const imageViewerState = ref({
  open: false,
  urls: [] as string[],
  index: 0
})

const progressValue = computed(() =>
  totalFileCount.value === 0 ? 0 : (finishFileCount.value / totalFileCount.value) * 100
)

const enableAddPaks = computed(() => unpackState.value.fileList !== '')
const enableExtract = computed(() => treeData.value !== null)
const fileTreeComponent = ref<InstanceType<typeof FileTree>>()
const fileNameTable = ref<{ openManager: () => void } | null>(null)
const texturePreviewEnabled = computed(
  () => settingsStore.settings.value?.preview?.showTexturePreview ?? true
)

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
      if (a.isDir !== b.isDir) {
        return a.isDir ? -1 : 1
      }

      return a.name.localeCompare(b.name)
    })
})

const explorerViewResetKey = computed(
  () =>
    `${treeData.value?.hash ?? 'root'}:${currentDirectoryKey.value}:${explorerSearchText.value}:${explorerLayoutMode.value}`
)
const explorerRenderers = computed<ExplorerRenderers>(() => ({
  getTexturePreview,
  getPreviewSurfaceStyle: getExplorerPreviewSurfaceStyle,
  getHeroIcon: getExplorerHeroIcon,
  getHeroIconStyle: getExplorerHeroIconStyle,
  getAccentStyle: getExplorerAccentStyle,
  getItemTypeLabel: getExplorerItemTypeLabel,
  getDirectoryCounts: getExplorerDirectoryCounts,
  getDetailText: getExplorerDetailText
}))
const explorerColumnLabels = computed<ExplorerColumnLabels>(() => ({
  name: t('unpack.columnName'),
  type: t('unpack.columnType'),
  size: t('unpack.columnSize'),
  details: t('unpack.columnDetails')
}))

const bringTargetKey = computed(() => {
  const entry = selectedEntry.value
  if (entry) {
    return entry.isDir ? entry.id : (entry.parentId ?? currentDirectoryKey.value)
  }

  return currentDirectoryKey.value
})
const currentDirectoryPath = computed(() => currentDirectory.value?.path ?? 'Root')
const statusText = computed(() => {
  if (unpackWorking.value) return 'Extracting'
  if (loadingTree.value) return 'Loading tree'
  if (!treeData.value) return 'Idle'
  return 'Completed'
})
const desktopMenuItems = computed<MenuGroup[]>(() => [
  {
    key: 'resources',
    label: t('menu.resources'),
    items: [
      {
        key: 'manage-path-lists',
        label: t('menu.managePathLists'),
        icon: Wrench,
        action: openPathListManager
      },
      {
        key: 'open-paks',
        label: t('menu.openPaks'),
        icon: FolderOpen,
        action: handleOpen
      }
    ]
  },
  {
    key: 'actions',
    label: t('menu.actions'),
    items: [
      {
        key: 'render-tree',
        label: t('menu.reloadTree'),
        icon: RefreshCw,
        action: handleToolbarRenderTree
      }
    ]
  }
])

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

function openPathListManager() {
  fileNameTable.value?.openManager()
}

watch(pakData, async () => {
  treeData.value = null
  currentDirectoryKey.value = ''
  selectedEntryKey.value = ''
  visibleExplorerEntries.value = []
  texturePreviewCache.value = {}
  texturePreviewPending.clear()
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
    treeFocusKey.value = ''
    visibleExplorerEntries.value = []
    return
  }

  currentDirectoryKey.value = explorerNodeMap.value.has(currentDirectoryKey.value)
    ? currentDirectoryKey.value
    : root.id

  if (!explorerNodeMap.value.has(treeFocusKey.value)) {
    treeFocusKey.value = root.id
  }
})

watch(
  () => [visibleExplorerEntries.value, texturePreviewEnabled.value] as const,
  ([entries, enabled]) => {
    if (!enabled) return
    void preloadTexturePreviews(entries)
  },
  { immediate: true }
)

watch(
  () => [currentDirectory.value?.id ?? '', treeData.value?.hash ?? ''] as const,
  () => {
    explorerLayoutMode.value = getDefaultExplorerLayout(currentDirectory.value)
  },
  { immediate: true }
)

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

watch(
  systemLogEntries,
  async (entries) => {
    const shouldStickToBottom = isConsoleNearBottom()
    const visibleEntries = isProductionBuild
      ? entries.filter((entry) => entry.level !== 'debug')
      : entries

    consoleLines.value = visibleEntries.slice(-160)

    if (!shouldStickToBottom) return

    await nextTick()
    scrollConsoleToBottom()
  },
  { immediate: true }
)

const updateFilter = () => {
  unpackState.value.filterText = unpackState.value.filterText.trim()
  filterTextApply.value = unpackState.value.filterText
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
  } catch (error) {
    ShowError(error)
  } finally {
    loadingTree.value = false
  }
}

const handleOrder = async () => {
  await reloadData()
}

async function handleCloseAll() {
  try {
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
        window.setProgressBar({ status: ProgressBarStatus.Normal, progress: 0 })
      } else if (event.event === 'workFinished') {
        unpackWorking.value = false
        if (finishFileCount.value !== totalFileCount.value) {
          finishFileCount.value = totalFileCount.value
        }
        window.setProgressBar({ status: ProgressBarStatus.None, progress: 0 })
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

async function dropInAddPaks(filePaths: string[]) {
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

let unlisten: UnlistenFn | undefined

function handleToolbarRenderTree() {
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
  ShowWarn(t('global.extractionTerminated'))
}

function handleNodeClick(data: TreeData) {
  treeFocusKey.value = data.id

  if (data.isDir) {
    openDirectory(data.id)
    selectedEntryKey.value = ''
    return
  }

  selectedEntryKey.value = data.id
  currentDirectoryKey.value = data.parentId ?? currentDirectoryKey.value
}

function bringSelectedEntryIntoTreeView() {
  const key = bringTargetKey.value
  if (!key) return

  treeFocusKey.value = key
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

function openParentDirectory() {
  const parentId = currentDirectory.value?.parentId
  if (!parentId) return
  openDirectory(parentId)
}

function toggleExplorerLayout() {
  explorerLayoutMode.value = explorerLayoutMode.value === 'tile' ? 'details' : 'tile'
  unpackState.value = {
    ...unpackState.value,
    explorerLayoutMode: explorerLayoutMode.value
  }
}

function collapseTree() {
  fileTreeComponent.value?.collapseAll()
}

function handleExplorerItemClick(item: ExplorerEntry) {
  selectedEntryKey.value = item.id
}

function handleVisibleExplorerItemsChange(items: ExplorerEntry[]) {
  visibleExplorerEntries.value = items
}

async function handleExplorerItemOpen(item: ExplorerEntry) {
  if (item.isDir) {
    openDirectory(item.id)
    return
  }

  selectedEntryKey.value = item.id

  const previewUrl = await ensureTexturePreview(item)
  if (!previewUrl) return

  imageViewerState.value = {
    open: true,
    urls: [previewUrl],
    index: 0
  }
}

function getExplorerTypeKey(item: ExplorerEntry) {
  return resolveExplorerFileTypeKey(item.name, item.isDir)
}

function canPreviewExplorerItem(item: ExplorerEntry) {
  return !item.isDir && texturePreviewEnabled.value && getExplorerTypeKey(item) === 'texture'
}

function getDefaultExplorerLayout(directory: ExplorerEntry | null): ExplorerLayoutMode {
  if (directory && directory.children.some((child) => !child.isDir && getExplorerTypeKey(child) === 'texture')) {
    return 'tile'
  }

  return unpackState.value.explorerLayoutMode ?? 'details'
}

function getExplorerTypeDefinition(item: ExplorerEntry) {
  return getExplorerFileTypeDefinition(getExplorerTypeKey(item))
}

function getExplorerTypeTheme(item: ExplorerEntry) {
  return getExplorerThemeForType(getExplorerTypeKey(item))
}

function getExplorerHeroIcon(item: ExplorerEntry) {
  return getExplorerTypeDefinition(item).icon
}

function getExplorerHeroIconStyle(item: ExplorerEntry): CSSProperties {
  return {
    color: getExplorerTypeTheme(item).hero
  }
}

function getExplorerPreviewSurfaceStyle(item: ExplorerEntry): CSSProperties {
  if (texturePreviewEnabled.value && getTexturePreview(item)) {
    return {
      background: 'color-mix(in oklch, var(--surface-toolbar) 92%, var(--surface-console))'
    }
  }

  const base = item.isDir
    ? 'color-mix(in oklch, var(--surface-toolbar) 82%, var(--surface-panel))'
    : 'color-mix(in oklch, var(--surface-toolbar) 90%, var(--surface-panel))'

  return {
    background: base
  }
}

function getExplorerAccentStyle(item: ExplorerEntry): CSSProperties {
  return {
    backgroundColor: getExplorerTypeTheme(item).accent
  }
}

function getExplorerItemTypeLabel(item: ExplorerEntry) {
  return getExplorerTypeDefinition(item).label
}

function getExplorerDirectoryCounts(item: ExplorerEntry): ExplorerDirectoryCounts {
  const folders = item.children.filter((child) => child.isDir).length
  return {
    folders,
    files: item.children.length - folders
  }
}

function getExplorerDetailText(item: ExplorerEntry) {
  if (item.isDir) {
    const counts = getExplorerDirectoryCounts(item)
    return t('unpack.directorySummary', counts)
  }

  return getExplorerSourceLabel(item.belongsTo)
}

function getExplorerSourceLabel(source?: string) {
  if (!source) {
    return '—'
  }

  const segments = source.split(/[\\/]/)
  return segments[segments.length - 1] ?? source
}

function getTexturePreview(item: ExplorerEntry) {
  return texturePreviewCache.value[item.id] ?? null
}

async function ensureTexturePreview(item: ExplorerEntry) {
  if (item.isDir || !item.hash || getExplorerTypeKey(item) !== 'texture') {
    return null
  }

  const cached = getTexturePreview(item)
  if (cached) {
    return cached
  }

  if (item.id in texturePreviewCache.value) {
    return texturePreviewCache.value[item.id]
  }

  if (texturePreviewPending.has(item.id)) {
    return waitForTexturePreview(item.id)
  }

  texturePreviewPending.add(item.id)

  try {
    const previewFile = await getPreviewFile(item.hash)
    const previewUrl = convertFileSrc(previewFile, 'asset')
    texturePreviewCache.value = {
      ...texturePreviewCache.value,
      [item.id]: previewUrl
    }
    return previewUrl
  } catch {
    texturePreviewCache.value = {
      ...texturePreviewCache.value,
      [item.id]: null
    }
    return null
  } finally {
    texturePreviewPending.delete(item.id)
  }
}

function waitForTexturePreview(itemId: string) {
  return new Promise<string | null>((resolve) => {
    const stop = watch(
      texturePreviewCache,
      (cache) => {
        if (!(itemId in cache)) return
        stop()
        resolve(cache[itemId] ?? null)
      },
      { flush: 'sync' }
    )
  })
}

async function preloadTexturePreviews(entries: ExplorerEntry[]) {
  const candidates = entries.filter(
    (entry) => !entry.isDir && entry.hash && !(entry.id in texturePreviewCache.value)
  )
  const concurrency = 6
  let cursor = 0

  const worker = async () => {
    while (cursor < candidates.length) {
      const item = candidates[cursor]
      cursor += 1
      if (!item?.hash) {
        continue
      }
      await ensureTexturePreview(item)
    }
  }

  await Promise.all(Array.from({ length: Math.min(concurrency, candidates.length) }, worker))
}

function closeImageViewer() {
  imageViewerState.value = {
    open: false,
    urls: [],
    index: 0
  }
}

function formatLogTime(value: string) {
  return new Date(value).toLocaleTimeString()
}

function isConsoleNearBottom() {
  const element = consoleContainer.value
  if (!element) return true

  const distanceToBottom = element.scrollHeight - element.scrollTop - element.clientHeight
  return distanceToBottom <= 24
}

function scrollConsoleToBottom() {
  const element = consoleContainer.value
  if (!element) return

  element.scrollTop = element.scrollHeight
}

function getLogLevelLabel(level: SystemLogLevel) {
  switch (level) {
    case 'error':
      return '[ERROR]'
    case 'warn':
      return '[WARN]'
    case 'info':
      return '[INFO]'
    case 'debug':
      return '[DEBUG]'
  }
}

function getLogLevelClass(level: SystemLogLevel) {
  switch (level) {
    case 'error':
      return 'text-destructive'
    case 'warn':
      return 'text-amber-400'
    case 'info':
      return 'text-sky-400'
    case 'debug':
      return 'text-emerald-400'
  }
}

function getLogMessageClass(level: SystemLogLevel) {
  switch (level) {
    case 'error':
      return 'text-destructive'
    case 'warn':
      return 'text-amber-200'
    case 'info':
      return 'text-foreground'
    case 'debug':
      return 'text-muted-foreground'
  }
}

onMounted(async () => {
  await startListenToDrop()
  await loadWorkRecords()
})

onUnmounted(async () => {
  await stopListenToDrop()
})
</script>
