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
                      :class="
                        segment.id === currentDirectoryKey ? 'font-medium text-foreground' : ''
                      "
                      @click="openDirectory(segment.id)"
                    >
                      {{ segment.label }}
                    </button>
                  </div>
                </div>

                <div class="relative min-h-0 flex-1">
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
                      <p class="section-copy">选择路径列表后，点击左侧刷新按钮生成 Explorer。</p>
                    </div>

                    <div
                      v-else
                      ref="explorerGridRef"
                      class="editor-scrollbar grid h-full grid-cols-[repeat(auto-fill,minmax(9rem,1fr))] gap-3 overflow-auto pr-1 content-start"
                    >
                      <button
                        v-for="item in explorerEntries"
                        :key="item.id"
                        type="button"
                        class="asset-tile-card group flex min-h-[13.5rem] flex-col overflow-hidden rounded-[0.4rem] bg-[#22242c] text-left shadow-[0_14px_28px_-24px_rgba(0,0,0,0.95)] transition-[background-color,box-shadow] duration-150 hover:bg-[#272a33] hover:shadow-[0_18px_30px_-24px_rgba(0,0,0,1)]"
                        :class="getExplorerCardClass(item)"
                        @click="handleExplorerItemClick(item)"
                        @dblclick="handleExplorerItemOpen(item)"
                      >
                        <div
                          class="relative flex h-30 items-center justify-center overflow-hidden bg-[#2a2d37] px-3 py-3"
                          :style="getExplorerPreviewSurfaceStyle(item)"
                        >
                          <template v-if="item.isDir">
                            <component
                              :is="getExplorerHeroIcon(item)"
                              class="asset-hero-icon size-14"
                              :style="getExplorerHeroIconStyle(item)"
                            />
                          </template>
                          <template v-else-if="texturePreviewEnabled && getTexturePreview(item)">
                            <el-image
                              :src="getTexturePreview(item) ?? undefined"
                              :alt="item.name"
                              fit="contain"
                              class="asset-tile-preview"
                              :preview-src-list="
                                getTexturePreview(item) ? [getTexturePreview(item)!] : []
                              "
                              :initial-index="0"
                              show-progress
                            >
                              <template #error>
                                <div class="flex h-full w-full items-center justify-center">
                                  <component
                                    :is="getExplorerHeroIcon(item)"
                                    class="asset-hero-icon size-12"
                                    :style="getExplorerHeroIconStyle(item)"
                                  />
                                </div>
                              </template>
                            </el-image>
                          </template>
                          <template v-else>
                            <component
                              :is="getExplorerHeroIcon(item)"
                              class="asset-hero-icon size-12"
                              :style="getExplorerHeroIconStyle(item)"
                            />
                          </template>
                        </div>

                        <div class="h-1 shrink-0" :style="getExplorerAccentStyle(item)" />

                        <div class="flex min-h-0 flex-1 flex-col px-3 py-2.5">
                          <p
                            class="line-clamp-2 min-h-[2.5rem] break-all text-[13px] font-semibold leading-5 text-foreground"
                          >
                            {{ item.name }}
                          </p>

                          <div
                            class="mt-auto flex items-center justify-between gap-3 pt-2 text-[10px] text-muted-foreground"
                          >
                            <span class="truncate">{{ getExplorerItemTypeLabel(item) }}</span>
                            <template v-if="item.isDir">
                              <span class="asset-counts shrink-0">
                                <span class="asset-count-chip">
                                  <Folder class="size-3" />
                                  {{ getExplorerDirectoryCounts(item).folders }}
                                </span>
                                <span class="asset-count-chip">
                                  <File class="size-3" />
                                  {{ getExplorerDirectoryCounts(item).files }}
                                </span>
                              </span>
                            </template>
                            <span v-else class="shrink-0">{{ item.sizeText }}</span>
                          </div>
                        </div>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </ResizablePanel>

            <ResizableHandle class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary" />

            <ResizablePanel :default-size="25" :max-size="42" :min-size="16">
              <div class="flex h-full min-w-0 flex-col bg-[#09090b]">
                <div
                  ref="consoleContainer"
                  class="editor-scrollbar min-h-0 min-w-0 flex-1 overflow-auto border border-border/60 bg-[#050507] px-3 py-2 font-mono text-[11px] leading-[1.45]"
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
  Download,
  File,
  FileArchive,
  FoldVertical,
  Filter,
  Folder,
  FolderTree,
  LocateFixed,
  PackageOpen,
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
import type { ExtractOptions, PakInfo, RenderTreeNode, UnpackProgressEvent } from '@/api/tauri/pak'
import { getPreviewFile } from '@/api/tauri/utils'
import FileTree, { type TreeData } from '@/components/FileTree.vue'
import FileNameTable from '@/components/FileNameTable/FileNameTable.vue'
import PakFiles from '@/components/PakFiles.vue'
import {
  defaultExplorerTypeThemes,
  getExplorerFileTypeDefinition,
  getExplorerResolvedTheme,
  resolveExplorerFileTypeKey
} from '@/lib/explorerTypeTheme'
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

type ExplorerDirectoryCounts = {
  folders: number
  files: number
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
const lastRefreshAt = ref<Date | null>(null)
const consoleLines = shallowRef<SystemLogEntry[]>([])
const consoleContainer = ref<HTMLElement | null>(null)
const explorerGridRef = ref<HTMLElement | null>(null)
const texturePreviewCache = ref<Record<string, string | null>>({})
const texturePreviewPending = new Set<string>()

const progressValue = computed(() =>
  totalFileCount.value === 0 ? 0 : (finishFileCount.value / totalFileCount.value) * 100
)

const enableAddPaks = computed(() => unpackState.value.fileList !== '')
const enableExtract = computed(() => treeData.value !== null)
const fileTreeComponent = ref<InstanceType<typeof FileTree>>()
const texturePreviewEnabled = computed(
  () => settingsStore.settings.value?.preview?.showTexturePreview ?? true
)
const explorerThemeOverrides = computed(
  () => settingsStore.settings.value?.preview?.explorerTypeThemes ?? defaultExplorerTypeThemes
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
  () => [explorerEntries.value, texturePreviewEnabled.value] as const,
  ([entries, enabled]) => {
    if (!enabled) return
    void preloadTexturePreviews(entries)
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
    lastRefreshAt.value = new Date()
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

function collapseTree() {
  fileTreeComponent.value?.collapseAll()
}

function handleExplorerItemClick(item: ExplorerEntry) {
  selectedEntryKey.value = item.id
}

function handleExplorerItemOpen(item: ExplorerEntry) {
  if (item.isDir) {
    openDirectory(item.id)
    return
  }

  selectedEntryKey.value = item.id
}

function getExplorerCardClass(item: ExplorerEntry) {
  if (selectedEntryKey.value !== item.id) {
    return 'ring-1 ring-transparent'
  }

  return 'bg-[#2a2d37] ring-1 ring-[#8ba5ff]/45 shadow-[0_0_0_1px_rgba(139,165,255,0.18),0_18px_40px_-28px_rgba(24,48,102,0.92)]'
}

function getExplorerTypeKey(item: ExplorerEntry) {
  return resolveExplorerFileTypeKey(item.name, item.isDir)
}

function getExplorerTypeDefinition(item: ExplorerEntry) {
  return getExplorerFileTypeDefinition(getExplorerTypeKey(item))
}

function getExplorerTypeTheme(item: ExplorerEntry) {
  return getExplorerResolvedTheme(getExplorerTypeKey(item), explorerThemeOverrides.value)
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
      background: '#1f2026'
    }
  }

  const base = item.isDir ? '#2b2d33' : '#262933'

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

function getTexturePreview(item: ExplorerEntry) {
  return texturePreviewCache.value[item.id] ?? null
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
      if (!item?.hash || texturePreviewPending.has(item.id)) {
        continue
      }

      texturePreviewPending.add(item.id)

      try {
        const previewFile = await getPreviewFile(item.hash)
        texturePreviewCache.value = {
          ...texturePreviewCache.value,
          [item.id]: convertFileSrc(previewFile, 'asset')
        }
      } catch {
        texturePreviewCache.value = {
          ...texturePreviewCache.value,
          [item.id]: null
        }
      } finally {
        texturePreviewPending.delete(item.id)
      }
    }
  }

  await Promise.all(Array.from({ length: Math.min(concurrency, candidates.length) }, worker))
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

<style scoped>
.asset-tile-card {
  position: relative;
  isolation: isolate;
}

.asset-tile-card > * {
  position: relative;
  z-index: 1;
}

.asset-tile-preview {
  height: 100%;
  width: 100%;
}

.asset-hero-icon {
  filter: drop-shadow(0 10px 18px rgb(0 0 0 / 0.28));
}

.asset-counts {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
}

.asset-count-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  color: rgb(255 255 255 / 0.74);
}

.asset-tile-preview :deep(.el-image__wrapper),
.asset-tile-preview :deep(.el-image__inner) {
  height: 100%;
  width: 100%;
}

.asset-tile-preview :deep(.el-image__inner) {
  object-fit: contain;
}

.asset-tile-preview :deep(.el-image__error) {
  background: transparent;
}
</style>
