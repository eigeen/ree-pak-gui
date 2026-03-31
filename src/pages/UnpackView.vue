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
                    <h2 class="section-title">{{ t('unpack.resourcesTitle') }}</h2>
                  </div>
                </div>
                <div class="mb-4">
                  <FileNameTable
                    v-model="unpackState.fileList"
                    :show-manage-button="false"
                    :show-manage-entry-in-selector="true"
                  />
                </div>
                <div class="mb-4">
                  <Button size="sm" class="w-full" :disabled="!canRenderTree" @click="doRender">
                    <RefreshCw class="size-4" :class="loadingTree ? 'animate-spin' : ''" />
                    {{ t('unpack.loadFileTree') }}
                  </Button>
                </div>
                <div class="min-h-0 flex-1">
                  <PakFiles
                    :pak-list="pakData"
                    @close="handleClose"
                    @close-all="handleCloseAll"
                    @open="handleOpen"
                    @order="handleOrder"
                    @show-properties="handlePakShowProperties"
                  />
                </div>
              </section>

              <section
                v-show="sidebarTab === 'tree'"
                class="flex min-h-0 flex-1 flex-col overflow-hidden"
              >
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
                <label class="text-xs mb-3 flex items-center gap-2 text-muted-foreground">
                  <Switch v-model="unpackState.filterUseRegex" />
                  <span>{{ t('unpack.regex') }}</span>
                </label>

                <div class="flex items-center h-8 min-h-8 justify-between px-0">
                  <div class="flex items-center gap-1.5">
                    <Button
                      size="icon-sm"
                      variant="ghost"
                      class="desktop-icon-button"
                      :disabled="!bringTargetKey"
                      :title="t('unpack.bringToTree')"
                      @click="bringSelectedEntryIntoTreeView"
                    >
                      <LocateFixed class="size-4" />
                    </Button>
                    <Button
                      size="icon-sm"
                      variant="ghost"
                      class="desktop-icon-button"
                      :disabled="!treeData"
                      :title="t('unpack.collapseAll')"
                      @click="collapseTree"
                    >
                      <FoldVertical class="size-4" />
                    </Button>
                  </div>
                  <div class="flex items-center gap-1.5">
                    <Button
                      size="icon-sm"
                      variant="ghost"
                      class="desktop-icon-button"
                      :disabled="selectedTreeExtractFiles.length === 0"
                      :title="t('unpack.extract')"
                      @click="doExtraction"
                    >
                      <Download class="size-4" />
                    </Button>
                    <Button
                      size="icon-sm"
                      variant="ghost"
                      class="desktop-icon-button"
                      :disabled="!showOverlay || loadingTree"
                      @click="doRender"
                    >
                      <RefreshCw class="size-4" :class="loadingTree ? 'animate-spin' : ''" />
                    </Button>
                  </div>
                </div>

                <div class="min-h-0 flex-1 pt-3">
                  <div v-if="!treeData" class="empty-state h-full border-0 bg-transparent">
                    <FileArchive class="size-8 text-muted-foreground" />
                    <p class="text-sm font-medium text-foreground">
                      {{
                        pakData.length === 0
                          ? t('unpack.emptyWaitLoadPaks')
                          : t('unpack.emptyWaitBuildTree')
                      }}
                    </p>
                    <p class="section-copy">
                      {{
                        pakData.length === 0
                          ? t('unpack.emptyHintLoadPrerequisite')
                          : t('unpack.emptyHintLoadTree')
                      }}
                    </p>
                  </div>

                  <FileTree
                    v-else
                    ref="fileTreeComponent"
                    :current-node-key="treeFocusKey"
                    :data="treePanelData"
                    class="h-full"
                    @node-click="handleNodeClick"
                    @node-contextmenu="handleTreeNodeContextMenu"
                    @background-contextmenu="handleTreeBackgroundContextMenu"
                  />
                </div>
              </section>
            </div>
          </aside>
        </ResizablePanel>

        <ResizableHandle class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary" />

        <ResizablePanel :default-size="76" :min-size="48">
          <ResizablePanelGroup direction="vertical">
            <ResizablePanel :default-size="75" :min-size="52">
              <UnpackExplorerPane
                v-model:search-text="explorerSearchText"
                :has-tree="Boolean(treeData)"
                :has-pak-data="pakData.length > 0"
                :layout-mode="explorerLayoutMode"
                :items="explorerEntries"
                :focused-key="focusedEntryKey"
                :checked-keys="checkedEntryKeys"
                :reset-key="explorerViewResetKey"
                :breadcrumb-segments="breadcrumbDisplaySegments"
                :current-directory-key="currentDirectoryKey"
                :can-go-parent-directory="Boolean(currentDirectory?.parentId)"
                :texture-preview-enabled="texturePreviewEnabled"
                :renderers="explorerRenderers"
                :column-labels="explorerColumnLabels"
                @open-directory="openDirectory"
                @open-parent-directory="openParentDirectory"
                @toggle-layout="toggleExplorerLayout"
                @item-click="handleExplorerItemClick"
                @item-check="handleExplorerItemCheck"
                @item-open="handleExplorerItemOpen"
                @item-contextmenu="handleExplorerItemContextMenu"
                @background-click="handleExplorerBackgroundClick"
                @background-contextmenu="handleExplorerBackgroundContextMenu"
                @visible-items-change="handleVisibleExplorerItemsChange"
              />
            </ResizablePanel>

            <ResizableHandle class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary" />

            <ResizablePanel :default-size="25" :max-size="42" :min-size="16">
              <SystemLogPanel empty-text="No system logs yet" />
            </ResizablePanel>
          </ResizablePanelGroup>
        </ResizablePanel>
      </ResizablePanelGroup>

      <div class="desktop-statusbar">
        <div class="flex items-center gap-3">
          <span>{{ statusText }}</span>
        </div>
        <div class="flex items-center gap-4">
          <span>{{ currentDirectoryPath }}</span>
        </div>
      </div>
    </div>

    <UnpackPropertiesDialog
      v-model:open="propertiesDialogOpen"
      :title="propertiesDialogTitle"
      :description="propertiesDialogDescription"
      :loading="propertiesDialogLoading"
      :sections="propertiesDialogSections"
      :empty-text="t('unpack.propertiesEmptyRaw')"
    />

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

    <AppCursorContextMenu
      :items="treeContextMenuItems"
      :open="treeContextMenuOpen"
      :x="treeContextMenuPosition.x"
      :y="treeContextMenuPosition.y"
      @update:open="treeContextMenuOpen = $event"
    />

    <AppCursorContextMenu
      :items="explorerContextMenuItems"
      :open="explorerContextMenuOpen"
      :x="explorerContextMenuPosition.x"
      :y="explorerContextMenuPosition.y"
      @update:open="explorerContextMenuOpen = $event"
    />
  </section>
</template>

<script setup lang="ts">
import {
  computed,
  onMounted,
  onUnmounted,
  ref,
  unref,
  watch,
  type CSSProperties,
  type Ref
} from 'vue'
import { Channel, convertFileSrc } from '@tauri-apps/api/core'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import { exists } from '@tauri-apps/plugin-fs'
import {
  Copy,
  Download,
  Eye,
  Info,
  FoldVertical,
  Filter,
  FileArchive,
  FolderOpen,
  FolderTree,
  LayoutGrid,
  List,
  LocateFixed,
  PackageOpen,
  RefreshCw,
  Wrench
} from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import {
  pak_close,
  pak_extract_all,
  pak_get_header,
  pak_list_all,
  pak_open,
  pak_read_file_tree_optimized,
  pak_terminate_extraction
} from '@/api/tauri/pak'
import type {
  ExtractFileInfo,
  ExtractMode,
  ExtractOptions,
  PakEntry,
  PakHeaderInfo,
  PakId,
  PakInfo,
  RenderTreeNode,
  UnpackProgressEvent
} from '@/api/tauri/pak'
import {
  exportTextureFiles,
  getPreviewFile,
  terminateTextureExport,
  type TextureExportFormat,
  type TextureExportProgressEvent
} from '@/api/tauri/utils'
import AppCursorContextMenu from '@/components/context-menu/AppCursorContextMenu.vue'
import FileTree from '@/components/FileTree.vue'
import type { MenuGroup } from '@/components/DesktopMenuBar.vue'
import FileNameTable from '@/components/FileNameTable/FileNameTable.vue'
import PageToolbar from '@/components/PageToolbar.vue'
import PakFiles from '@/components/PakFiles.vue'
import SystemLogPanel from '@/components/SystemLogPanel.vue'
import UnpackSidebarTabs, {
  type UnpackSidebarTabItem
} from '@/components/unpack/UnpackSidebarTabs.vue'
import UnpackExplorerPane from '@/components/unpack/UnpackExplorerPane.vue'
import UnpackPropertiesDialog, {
  type PropertySection
} from '@/components/unpack/UnpackPropertiesDialog.vue'
import {
  getExplorerFileTypeDefinition,
  getExplorerThemeForType,
  resolveExplorerFileTypeKey
} from '@/lib/explorerTypeTheme'
import type { ContextMenuEntry } from '@/lib/contextMenu'
import type {
  ExplorerColumnLabels,
  ExplorerDirectoryCounts,
  ExplorerEntry,
  ExplorerLayoutMode,
  ExplorerRenderers
} from '@/lib/unpackExplorer'
import {
  appendRangeToCheckedKeys,
  getOrderedCheckedItems,
  replaceCheckedKeysWithSingle,
  toggleCheckedKey
} from '@/lib/unpackExplorerSelection'
import {
  buildDirectoryTreeData,
  buildTreeData,
  createTreeFilter,
  filterTreeData,
  type TreeData
} from '@/lib/unpackTree'
import {
  ensureTaskProgressIdle,
  finishTaskProgress,
  tryStartTaskProgress,
  updateTaskProgress,
  useTaskProgressState
} from '@/service/taskProgress'
import { fileListService } from '@/service/filelist'
import { useSettingsStore, type AppSettings } from '@/store/settings'
import { useWorkStore } from '@/store/work'
import { ShowError, ShowInfo, ShowWarn } from '@/utils/message'
import {
  getSelectedItemRelativeRoot,
  normalizeDisplayPath,
  splitNormalizedPath
} from '@/utils/path'
import { Button } from '@/components/ui/button'
import { DenseInput } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { ElImageViewer } from 'element-plus'

type UnpackState = {
  fileList: string
  paks: string[]
  filterText: string
  filterUseRegex: boolean
  explorerLayoutMode: ExplorerLayoutMode
}

type SidebarTab = 'resources' | 'tree'
type ExplorerContextMenuKind = 'item' | 'background'
type TreeContextMenuKind = 'node' | 'background'
type PropertyTarget =
  | { kind: 'directory'; node: ExplorerEntry }
  | { kind: 'file'; node: ExplorerEntry }
  | { kind: 'pak'; pak: PakInfo }

const EXPLORER_ROOT_ID = '__explorer_root__'

const { t } = useI18n()
const workStore = useWorkStore()
const settingsStore = useSettingsStore()
const settings = computed(() => unref(settingsStore.settings as unknown as Ref<AppSettings>))

const unpackState = computed({
  get: () => workStore.unpack as unknown as UnpackState,
  set: (value: UnpackState) => {
    ;(workStore as any).unpack = value
  }
})

const filterTextApply = ref('')
const explorerSearchText = ref('')
const sidebarTab = ref<SidebarTab>('resources')
const sidebarTabs = computed<UnpackSidebarTabItem[]>(() => [
  {
    value: 'resources',
    label: t('unpack.resourcesTitle'),
    icon: PackageOpen
  },
  {
    value: 'tree',
    label: t('unpack.treeTab'),
    icon: FolderTree
  }
])
const pakData = ref<PakInfo[]>([])
const initialLoaded = ref(false)
const treeData = ref<RenderTreeNode[] | null>(null)
const showOverlay = ref(false)
const loadingTree = ref(false)
const currentDirectoryKey = ref('')
const treeFocusKey = ref('')
const focusedEntryKey = ref('')
const checkedEntryKeys = ref<string[]>([])
const selectionAnchorKey = ref('')
const visibleExplorerEntries = ref<ExplorerEntry[]>([])
const texturePreviewCache = ref<Record<string, string | null>>({})
const texturePreviewPending = new Set<string>()
const explorerLayoutMode = ref<ExplorerLayoutMode>('details')
const explorerContextMenuKind = ref<ExplorerContextMenuKind>('background')
const explorerContextMenuTarget = ref<ExplorerEntry | null>(null)
const explorerContextMenuOpen = ref(false)
const explorerContextMenuPosition = ref({ x: 0, y: 0 })
const treeContextMenuKind = ref<TreeContextMenuKind>('background')
const treeContextMenuTarget = ref<TreeData | null>(null)
const treeContextMenuOpen = ref(false)
const treeContextMenuPosition = ref({ x: 0, y: 0 })
const pakHeaderCache = ref<Record<string, PakHeaderInfo>>({})
const propertiesDialogOpen = ref(false)
const propertiesDialogLoading = ref(false)
const propertiesDialogTitle = ref('')
const propertiesDialogDescription = ref('')
const propertiesDialogSections = ref<PropertySection[]>([])
const propertyTarget = ref<PropertyTarget | null>(null)
const imageViewerState = ref({
  open: false,
  urls: [] as string[],
  index: 0
})
const taskProgress = useTaskProgressState()

const canRenderTree = computed(
  () => Boolean(unpackState.value.fileList) && pakData.value.length > 0 && !loadingTree.value
)
const fileTreeComponent = ref<InstanceType<typeof FileTree>>()
const fileNameTable = ref<{ openManager: () => void } | null>(null)
const texturePreviewEnabled = computed(() => settings.value?.preview?.showTexturePreview ?? true)
const extractMode = computed<ExtractMode>(() =>
  settings.value?.unpack?.extractAbsolutePath ? 'absolutePath' : 'relativePath'
)
const fullTreeData = computed<TreeData[]>(() =>
  treeData.value ? buildTreeData(treeData.value) : []
)
const treeFilter = computed(() =>
  createTreeFilter(filterTextApply.value, unpackState.value.filterUseRegex)
)
const filteredFullTreeData = computed(() => filterTreeData(fullTreeData.value, treeFilter.value))
const treePanelData = computed(() => buildDirectoryTreeData(filteredFullTreeData.value))

const explorerRoot = computed<ExplorerEntry | null>(() => {
  if (!treeData.value) {
    return null
  }

  return buildExplorerRoot(filteredFullTreeData.value)
})

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

const checkedEntryKeySet = computed(() => new Set(checkedEntryKeys.value))
const focusedEntry = computed(() =>
  focusedEntryKey.value ? explorerNodeMap.value.get(focusedEntryKey.value) : undefined
)
const selectedTreeEntry = computed(() =>
  treeFocusKey.value ? explorerNodeMap.value.get(treeFocusKey.value) : undefined
)
const selectedTreeExtractFiles = computed(() => {
  const entry = selectedTreeEntry.value
  return entry?.isDir ? collectExtractFilesFromEntry(entry, 'relativePath') : []
})
const orderedCheckedExplorerEntries = computed(() =>
  getOrderedCheckedItems(explorerEntries.value, checkedEntryKeys.value)
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
const pakFileNameMap = computed(() => {
  const map = new Map<string, string>()

  for (const pak of pakData.value) {
    const segments = splitNormalizedPath(pak.path)
    const fileName = segments[segments.length - 1] ?? pak.path
    map.set(pak.id, fileName)
  }

  return map
})

const explorerViewResetKey = computed(
  () =>
    `${treePanelData.value.length}:${filterTextApply.value}:${unpackState.value.filterUseRegex}:${currentDirectoryKey.value}:${explorerSearchText.value}:${explorerLayoutMode.value}`
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

function buildDirectoryContextMenuEntries(
  item: ExplorerEntry,
  options: {
    keyPrefix: string
    actionEntries?: ExplorerEntry[]
    includeLocateInTree?: boolean
  }
): ContextMenuEntry[] {
  const actionEntries = options.actionEntries ?? [item]
  const extractFiles = collectExtractFilesFromEntries(actionEntries)
  const extractDirectoryFiles = collectExtractFilesFromEntries(actionEntries, 'relativePath')
  const textureFiles = collectTextureFilesFromEntries(actionEntries, 'relativePath')
  const pathTargets = actionEntries.map((entry) => entry.path)
  const entries: ContextMenuEntry[] = [
    {
      type: 'action',
      key: `${options.keyPrefix}-open-directory`,
      label: t('unpack.openDirectory'),
      icon: FolderOpen,
      action: () => openDirectory(item.id)
    }
  ]

  if (options.includeLocateInTree ?? false) {
    entries.push({
      type: 'action',
      key: `${options.keyPrefix}-locate-tree`,
      label: t('unpack.locateInTree'),
      icon: LocateFixed,
      disabled: !item.parentId,
      action: () => bringEntryIntoTreeView(item)
    })
  }

  entries.push(
    {
      type: 'action',
      key: `${options.keyPrefix}-properties`,
      label: t('unpack.viewProperties'),
      icon: Info,
      action: () => void openPropertiesDialog(item)
    },
    {
      type: 'separator',
      key: `${options.keyPrefix}-item-separator`
    },
    {
      type: 'action',
      key: `${options.keyPrefix}-extract-full-path`,
      label: t('unpack.exportFullPath'),
      icon: FolderTree,
      disabled: extractFiles.length === 0,
      action: () => void extractFilesWithDialog(extractFiles, 'absolutePath')
    },
    {
      type: 'action',
      key: `${options.keyPrefix}-extract-current-path`,
      label: t('unpack.exportCurrentPath'),
      icon: Download,
      disabled: extractFiles.length === 0,
      action: () => void extractFilesWithDialog(extractDirectoryFiles, 'relativePath')
    },
    {
      type: 'action',
      key: `${options.keyPrefix}-copy-path`,
      label: t('unpack.copyPath'),
      icon: Copy,
      shortcut: 'Ctrl+C',
      action: () => void copyPaths(pathTargets)
    }
  )

  if (textureFiles.length > 0) {
    entries.push({
      type: 'submenu',
      key: `${options.keyPrefix}-other-export-actions`,
      label: t('unpack.otherExportActions'),
      icon: Download,
      children: [
        {
          type: 'action',
          key: `${options.keyPrefix}-export-texture-dds`,
          label: t('unpack.exportTexturesAsDds'),
          action: () => void exportTexturesWithDialog(textureFiles, 'dds')
        },
        {
          type: 'action',
          key: `${options.keyPrefix}-export-texture-png`,
          label: t('unpack.exportTexturesAsPng'),
          action: () => void exportTexturesWithDialog(textureFiles, 'png')
        }
      ]
    })
  }

  return entries
}

const explorerContextMenuItems = computed<ContextMenuEntry[]>(() => {
  if (!treeData.value) {
    return []
  }

  if (explorerContextMenuKind.value === 'background') {
    return [
      {
        type: 'submenu',
        key: 'explorer-layout',
        label: t('unpack.layoutMenu'),
        icon: explorerLayoutMode.value === 'tile' ? LayoutGrid : List,
        children: [
          {
            type: 'action',
            key: 'explorer-layout-details',
            label: t('unpack.layoutDetails'),
            icon: List,
            disabled: explorerLayoutMode.value === 'details',
            action: () => setExplorerLayout('details')
          },
          {
            type: 'action',
            key: 'explorer-layout-tile',
            label: t('unpack.layoutTile'),
            icon: LayoutGrid,
            disabled: explorerLayoutMode.value === 'tile',
            action: () => setExplorerLayout('tile')
          }
        ]
      },
      {
        type: 'action',
        key: 'explorer-open-parent',
        label: t('unpack.openParentDirectory'),
        icon: FolderOpen,
        disabled: !currentDirectory.value?.parentId,
        action: openParentDirectory
      },
      {
        type: 'separator',
        key: 'explorer-background-separator'
      },
      {
        type: 'action',
        key: 'explorer-refresh-tree',
        label: t('menu.reloadTree'),
        icon: RefreshCw,
        disabled: !canRenderTree.value,
        action: handleToolbarRenderTree
      }
    ]
  }

  const item = explorerContextMenuTarget.value
  if (!item) {
    return []
  }

  if (item.isDir) {
    return buildDirectoryContextMenuEntries(item, {
      keyPrefix: 'explorer-directory',
      actionEntries: getExplorerBatchActionEntries(item),
      includeLocateInTree: true
    })
  }

  const actionEntries = getExplorerBatchActionEntries(item)
  const extractFiles = collectExtractFilesFromEntries(actionEntries)
  const textureFiles = collectTextureFilesFromEntries(actionEntries, 'relativePath')
  const pathTargets = getExplorerCopyPathTargets(item)
  const canPreview = canPreviewExplorerItem(item)

  const entries: ContextMenuEntry[] = [
    {
      type: 'action',
      key: 'explorer-primary-open',
      label: t('unpack.previewItem'),
      icon: Eye,
      disabled: !canPreview,
      action: () => {
        void handleExplorerItemOpen(item)
      }
    },
    {
      type: 'action',
      key: 'explorer-locate-tree',
      label: t('unpack.locateInTree'),
      icon: LocateFixed,
      disabled: !item.isDir && !item.parentId,
      action: () => bringEntryIntoTreeView(item)
    },
    {
      type: 'action',
      key: 'explorer-properties',
      label: t('unpack.viewProperties'),
      icon: Info,
      action: () => void openPropertiesDialog(item)
    },
    {
      type: 'separator',
      key: 'explorer-item-separator'
    },
    {
      type: 'action',
      key: 'explorer-extract-full-path',
      label: t('unpack.exportFullPath'),
      icon: FolderTree,
      disabled: extractFiles.length === 0,
      action: () => void extractFilesWithDialog(extractFiles, 'absolutePath')
    },
    {
      type: 'action',
      key: 'explorer-extract-current-path',
      label: t('unpack.exportCurrentPath'),
      icon: Download,
      disabled: extractFiles.length === 0,
      action: () => void extractFilesWithDialog(extractFiles, 'relativePath')
    },
    {
      type: 'action',
      key: 'explorer-copy-path',
      label: t('unpack.copyPath'),
      icon: Copy,
      shortcut: 'Ctrl+C',
      action: () => void copyPaths(pathTargets)
    }
  ]

  if (textureFiles.length > 0) {
    entries.push({
      type: 'submenu',
      key: 'explorer-other-export-actions',
      label: t('unpack.otherExportActions'),
      icon: Download,
      children: [
        {
          type: 'action',
          key: 'explorer-export-texture-dds',
          label: t('unpack.exportTexturesAsDds'),
          action: () => void exportTexturesWithDialog(textureFiles, 'dds')
        },
        {
          type: 'action',
          key: 'explorer-export-texture-png',
          label: t('unpack.exportTexturesAsPng'),
          action: () => void exportTexturesWithDialog(textureFiles, 'png')
        }
      ]
    })
  }

  return entries
})
const treeContextMenuItems = computed<ContextMenuEntry[]>(() => {
  if (!treeData.value) {
    return []
  }

  if (treeContextMenuKind.value === 'background') {
    return [
      {
        type: 'action',
        key: 'tree-refresh',
        label: t('menu.reloadTree'),
        icon: RefreshCw,
        disabled: !canRenderTree.value,
        action: handleToolbarRenderTree
      },
      {
        type: 'action',
        key: 'tree-collapse',
        label: t('unpack.collapseAll'),
        icon: FoldVertical,
        action: collapseTree
      }
    ]
  }

  const node = treeContextMenuTarget.value
  if (!node) {
    return []
  }

  const entry = explorerNodeMap.value.get(node.id)
  if (!entry?.isDir) {
    return []
  }

  return buildDirectoryContextMenuEntries(entry, {
    keyPrefix: 'tree-directory'
  })
})

const bringTargetKey = computed(() => {
  const entry = focusedEntry.value
  if (entry) {
    return entry.isDir ? entry.id : (entry.parentId ?? currentDirectoryKey.value)
  }

  if (currentDirectoryKey.value && currentDirectoryKey.value !== EXPLORER_ROOT_ID) {
    return currentDirectoryKey.value
  }

  return explorerRoot.value?.children[0]?.id ?? ''
})
const currentDirectoryPath = computed(() =>
  currentDirectory.value?.path ? currentDirectory.value.path : t('unpack.rootLabel')
)
const statusText = computed(() => {
  if (taskProgress.working) return taskProgress.title
  if (loadingTree.value) return t('unpack.loadingTree')
  if (!treeData.value) return t('unpack.idle')
  return t('unpack.completed')
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
    if (cursor.id === EXPLORER_ROOT_ID) {
      break
    }

    const labels = splitBreadcrumbLabel(cursor.name)
    for (let i = labels.length - 1; i >= 0; i -= 1) {
      segments.unshift({ id: cursor.id, label: labels[i] ?? cursor.name })
    }
    cursor = cursor.parentId ? (explorerNodeMap.value.get(cursor.parentId) ?? null) : null
  }

  return segments
})

function splitBreadcrumbLabel(label: string): string[] {
  const parts = label.split(/\s*\/\s*/).filter((part) => part.length > 0)
  return parts.length > 0 ? parts : [label]
}

function openPathListManager() {
  fileNameTable.value?.openManager()
}

function setExplorerFocus(key: string) {
  focusedEntryKey.value = key
}

function clearExplorerSelection(options: { clearContextMenuTarget?: boolean } = {}) {
  focusedEntryKey.value = ''
  checkedEntryKeys.value = []
  selectionAnchorKey.value = ''
  explorerContextMenuOpen.value = false

  if (options.clearContextMenuTarget ?? true) {
    explorerContextMenuTarget.value = null
  }
}

function getExplorerBatchActionEntries(fallbackItem: ExplorerEntry) {
  const batchEntries = orderedCheckedExplorerEntries.value
  return batchEntries.length > 0 ? batchEntries : [fallbackItem]
}

function getExplorerCopyPathTargets(fallbackItem: ExplorerEntry) {
  return getExplorerBatchActionEntries(fallbackItem).map((entry) => entry.path)
}

function setExplorerLayout(mode: ExplorerLayoutMode) {
  explorerLayoutMode.value = mode
  unpackState.value = {
    ...unpackState.value,
    explorerLayoutMode: mode
  }
}

watch(pakData, async () => {
  treeData.value = null
  currentDirectoryKey.value = ''
  clearExplorerSelection()
  explorerContextMenuTarget.value = null
  explorerContextMenuOpen.value = false
  treeContextMenuTarget.value = null
  treeContextMenuOpen.value = false
  pakHeaderCache.value = {}
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
    clearExplorerSelection()
    visibleExplorerEntries.value = []
    return
  }

  currentDirectoryKey.value = explorerNodeMap.value.has(currentDirectoryKey.value)
    ? currentDirectoryKey.value
    : root.id

  if (!explorerNodeMap.value.has(treeFocusKey.value)) {
    treeFocusKey.value = root.children[0]?.id ?? ''
  }

  if (!explorerNodeMap.value.has(focusedEntryKey.value)) {
    clearExplorerSelection({ clearContextMenuTarget: true })
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
  () =>
    [
      currentDirectory.value?.id ?? '',
      filteredFullTreeData.value.length,
      filterTextApply.value,
      unpackState.value.filterUseRegex
    ] as const,
  () => {
    explorerLayoutMode.value = getDefaultExplorerLayout(currentDirectory.value)
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
  if (loadingTree.value) {
    return
  }

  loadingTree.value = true
  try {
    const file = fileListService.getFileByIdent(unpackState.value.fileList)
    if (!file) {
      throw new Error(`Name list file not found: ${unpackState.value.fileList}`)
    }

    await fileListService.loadFilePathList(file.source.filePath)
    treeData.value = await pak_read_file_tree_optimized()
    sidebarTab.value = 'tree'
    showOverlay.value = false
  } catch (error) {
    ShowError(error)
  } finally {
    loadingTree.value = false
  }
}

const handleOrder = async (order: PakId[]) => {
  const pakMap = new Map(pakData.value.map((pak) => [pak.id, pak] as const))
  const orderedPaks = order
    .map((id) => pakMap.get(id))
    .filter((pak): pak is PakInfo => Boolean(pak))

  if (orderedPaks.length !== pakData.value.length) {
    await reloadData()
    return
  }

  pakData.value = orderedPaks
  unpackState.value = {
    ...unpackState.value,
    paks: orderedPaks.map((pak) => pak.path)
  }
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
  await extractFilesWithDialog(selectedTreeExtractFiles.value, 'relativePath')
}

async function extractFilesWithDialog(extractFiles: ExtractFileInfo[], mode: ExtractMode) {
  if (!ensureTaskProgressIdle(t('global.taskBusy'))) {
    return
  }

  try {
    if (extractFiles.length === 0) {
      ShowWarn(t('unpack.noExtractableFiles'))
      return
    }

    let selected = await dialogOpen({
      directory: true,
      multiple: false
    })

    if (!selected) return
    if (Array.isArray(selected)) selected = selected[0]

    const options: ExtractOptions = {
      outputPath: selected as string,
      override: true,
      mode,
      extractAll: false,
      extractFiles
    }

    const onEvent = new Channel<UnpackProgressEvent>()
    const taskId = tryStartTaskProgress({
      taskId: 'unpack-extract',
      title: t('unpack.extractingFiles'),
      progressLabel: t('unpack.extracting'),
      runningDescription: t('unpack.processing'),
      successDescription: t('unpack.done'),
      terminatedDescription: t('unpack.taskStopped'),
      closeLabel: t('unpack.close'),
      terminateLabel: t('unpack.terminate'),
      confirmTitle: t('unpack.confirmTermination'),
      confirmDescription: t('unpack.confirmTerminationText'),
      busyMessage: t('global.taskBusy'),
      onTerminate: async () => {
        await pak_terminate_extraction()
        ShowWarn(t('global.extractionTerminated'))
      }
    })
    if (!taskId) return

    onEvent.onmessage = (event) => {
      if (event.event === 'workStart') {
        updateTaskProgress(taskId, {
          totalFileCount: event.data.count,
          finishFileCount: 0,
          currentFile: '',
          description: t('unpack.processing')
        })
      } else if (event.event === 'workFinished') {
        finishTaskProgress(taskId, {
          status: 'success',
          finishFileCount: taskProgress.totalFileCount
        })
      } else if (event.event === 'fileDone') {
        updateTaskProgress(taskId, {
          finishFileCount: event.data.finishCount,
          currentFile: event.data.path
        })
      } else if (event.event === 'error') {
        finishTaskProgress(taskId, {
          status: 'error',
          errorMessage: event.data.error,
          currentFile: event.data.error,
          description: t('unpack.taskStopped')
        })
      }
    }

    await pak_extract_all(options, onEvent)
  } catch (error) {
    finishTaskProgress('unpack-extract', {
      status: 'error',
      errorMessage: String(error),
      currentFile: String(error),
      description: t('unpack.taskStopped')
    })
    ShowError(error)
  }
}

async function exportTexturesWithDialog(files: ExtractFileInfo[], format: TextureExportFormat) {
  if (!ensureTaskProgressIdle(t('global.taskBusy'))) {
    return
  }

  try {
    if (files.length === 0) {
      ShowWarn(t('unpack.noExportableTextures'))
      return
    }

    let selected = await dialogOpen({
      directory: true,
      multiple: false
    })

    if (!selected) return
    if (Array.isArray(selected)) selected = selected[0]

    const onEvent = new Channel<TextureExportProgressEvent>()
    const taskId = tryStartTaskProgress({
      taskId: `texture-export-${format}`,
      title: t('unpack.exportingTextures'),
      progressLabel: t('unpack.exporting'),
      runningDescription: t('unpack.processing'),
      successDescription: t('unpack.done'),
      terminatedDescription: t('unpack.taskStopped'),
      closeLabel: t('unpack.close'),
      terminateLabel: t('unpack.terminate'),
      confirmTitle: t('unpack.confirmTermination'),
      confirmDescription: t('unpack.confirmTerminationText'),
      busyMessage: t('global.taskBusy'),
      onTerminate: async () => {
        await terminateTextureExport()
        ShowWarn(t('unpack.taskStopped'))
      }
    })
    if (!taskId) return

    onEvent.onmessage = (event) => {
      if (event.event === 'workStart') {
        updateTaskProgress(taskId, {
          totalFileCount: event.data.count,
          finishFileCount: 0,
          currentFile: '',
          description: t('unpack.processing')
        })
      } else if (event.event === 'fileDone') {
        updateTaskProgress(taskId, {
          finishFileCount: event.data.finishCount,
          currentFile: event.data.path
        })
      } else if (event.event === 'workFinished') {
        finishTaskProgress(taskId, {
          status: 'success',
          finishFileCount: taskProgress.totalFileCount
        })
      } else if (event.event === 'error') {
        finishTaskProgress(taskId, {
          status: 'error',
          errorMessage: event.data.error,
          currentFile: event.data.error,
          description: t('unpack.taskStopped')
        })
      }
    }

    const exported = await exportTextureFiles(
      {
        outputPath: selected as string,
        format,
        files
      },
      onEvent
    )

    finishTaskProgress(taskId, {
      status: 'success',
      finishFileCount: taskProgress.totalFileCount
    })

    ShowInfo(
      t('unpack.textureExported', {
        count: exported,
        format: format.toUpperCase()
      })
    )
  } catch (error) {
    finishTaskProgress(`texture-export-${format}`, {
      status: 'error',
      errorMessage: String(error),
      currentFile: String(error),
      description: t('unpack.taskStopped')
    })
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
  if (!unpackState.value.fileList || pakData.value.length === 0 || loadingTree.value) return
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

function handleNodeClick(data: TreeData) {
  explorerContextMenuOpen.value = false
  treeContextMenuOpen.value = false
  treeFocusKey.value = data.id
  treeContextMenuKind.value = 'node'
  treeContextMenuTarget.value = data

  if (data.isDir) {
    openDirectory(data.id)
    return
  }

  setExplorerFocus(data.id)
  checkedEntryKeys.value = []
  selectionAnchorKey.value = ''
  currentDirectoryKey.value = data.parentId ?? currentDirectoryKey.value
}

function bringSelectedEntryIntoTreeView() {
  const key = bringTargetKey.value
  if (!key) return

  treeFocusKey.value = key
  fileTreeComponent.value?.bringNodeIntoView(key)
}

function handleTreeNodeContextMenu(data: TreeData, _node: unknown, event: MouseEvent) {
  event.preventDefault()
  explorerContextMenuOpen.value = false
  treeContextMenuKind.value = 'node'
  treeContextMenuTarget.value = data
  treeFocusKey.value = data.id
  openDirectory(data.id)
  treeContextMenuPosition.value = {
    x: event.clientX,
    y: event.clientY
  }
  treeContextMenuOpen.value = true
}

function handleTreeBackgroundContextMenu(event: MouseEvent) {
  event.preventDefault()
  explorerContextMenuOpen.value = false
  treeContextMenuKind.value = 'background'
  treeContextMenuTarget.value = null
  treeContextMenuPosition.value = {
    x: event.clientX,
    y: event.clientY
  }
  treeContextMenuOpen.value = true
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

function buildExplorerTree(node: TreeData, parentPath = '', parentId?: string): ExplorerEntry {
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
    compressedSize: node.compressedSize,
    uncompressedSize: node.uncompressedSize,
    isCompressed: node.isCompressed,
    sizeText: formatSize(
      node.uncompressedSize !== undefined
        ? node.isCompressed
          ? node.uncompressedSize
          : node.compressedSize
        : 0
    ),
    children: node.children.map((child) => buildExplorerTree(child, path, id)),
    belongsTo: node.belongsTo
  }
}

function buildExplorerRoot(nodes: TreeData[]): ExplorerEntry {
  return {
    id: EXPLORER_ROOT_ID,
    name: '',
    label: t('unpack.rootLabel'),
    path: '',
    isDir: true,
    compressedSize: 0,
    uncompressedSize: 0,
    isCompressed: false,
    sizeText: '',
    children: nodes.map((node) => buildExplorerTree(node, '', EXPLORER_ROOT_ID)),
    belongsTo: undefined
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
  treeContextMenuOpen.value = false
  explorerContextMenuOpen.value = false
  clearExplorerSelection({ clearContextMenuTarget: true })
}

function openParentDirectory() {
  const parentId = currentDirectory.value?.parentId
  if (!parentId) return
  openDirectory(parentId)
}

function toggleExplorerLayout() {
  setExplorerLayout(explorerLayoutMode.value === 'tile' ? 'details' : 'tile')
}

function collapseTree() {
  fileTreeComponent.value?.collapseAll()
}

function handleExplorerItemClick(item: ExplorerEntry, event: MouseEvent) {
  if (event.shiftKey && selectionAnchorKey.value) {
    setExplorerFocus(item.id)
    checkedEntryKeys.value = appendRangeToCheckedKeys(
      checkedEntryKeys.value,
      explorerEntries.value,
      selectionAnchorKey.value,
      item.id
    )
    selectionAnchorKey.value = item.id
    return
  }

  setExplorerFocus(item.id)

  if (event.detail > 1) {
    if (!checkedEntryKeySet.value.has(item.id)) {
      checkedEntryKeys.value = [...checkedEntryKeys.value, item.id]
    }
    return
  }

  checkedEntryKeys.value = toggleCheckedKey(checkedEntryKeys.value, item.id)
  selectionAnchorKey.value = item.id
}

function handleExplorerItemCheck(item: ExplorerEntry, checked: boolean) {
  setExplorerFocus(item.id)
  checkedEntryKeys.value = checked
    ? checkedEntryKeySet.value.has(item.id)
      ? checkedEntryKeys.value
      : [...checkedEntryKeys.value, item.id]
    : checkedEntryKeys.value.filter((checkedKey) => checkedKey !== item.id)
  selectionAnchorKey.value = item.id
}

function handleExplorerItemContextMenu(item: ExplorerEntry, event: MouseEvent) {
  event.preventDefault()
  treeContextMenuOpen.value = false
  if (!checkedEntryKeySet.value.has(item.id)) {
    checkedEntryKeys.value = replaceCheckedKeysWithSingle(item.id)
  }

  explorerContextMenuKind.value = 'item'
  explorerContextMenuTarget.value = item
  explorerContextMenuPosition.value = {
    x: event.clientX,
    y: event.clientY
  }
  explorerContextMenuOpen.value = true
  setExplorerFocus(item.id)
}

function handleExplorerBackgroundClick() {
  treeContextMenuOpen.value = false
  explorerContextMenuOpen.value = false
  clearExplorerSelection()
}

function handleExplorerBackgroundContextMenu(event: MouseEvent) {
  event.preventDefault()
  treeContextMenuOpen.value = false
  explorerContextMenuKind.value = 'background'
  explorerContextMenuTarget.value = null
  explorerContextMenuPosition.value = {
    x: event.clientX,
    y: event.clientY
  }
  explorerContextMenuOpen.value = true
}

function handleVisibleExplorerItemsChange(items: ExplorerEntry[]) {
  visibleExplorerEntries.value = items
}

async function handleExplorerItemOpen(item: ExplorerEntry) {
  setExplorerFocus(item.id)
  if (!checkedEntryKeySet.value.has(item.id)) {
    checkedEntryKeys.value = [...checkedEntryKeys.value, item.id]
  }
  selectionAnchorKey.value = item.id

  if (item.isDir) {
    openDirectory(item.id)
    return
  }

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

function isTextureEntry(item: ExplorerEntry) {
  return !item.isDir && getExplorerTypeKey(item) === 'texture'
}

function canPreviewExplorerItem(item: ExplorerEntry) {
  return texturePreviewEnabled.value && isTextureEntry(item)
}

function getDefaultExplorerLayout(directory: ExplorerEntry | null): ExplorerLayoutMode {
  if (
    directory &&
    directory.children.some((child) => !child.isDir && getExplorerTypeKey(child) === 'texture')
  ) {
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
      background: 'color-mix(in srgb, var(--surface-toolbar) 92%, var(--surface-console))'
    }
  }

  const base = item.isDir
    ? 'color-mix(in srgb, var(--surface-toolbar) 82%, var(--surface-panel))'
    : 'color-mix(in srgb, var(--surface-toolbar) 90%, var(--surface-panel))'

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
    return t('unpack.sourceEmpty')
  }

  return t('unpack.sourceLabel', { source: pakFileNameMap.value.get(source) ?? source })
}

async function getPakHeaderInfo(path: string) {
  const cached = pakHeaderCache.value[path]
  if (cached) {
    return cached
  }

  const header = await pak_get_header(path)
  pakHeaderCache.value = {
    ...pakHeaderCache.value,
    [path]: header
  }
  return header
}

function findPakInfo(id?: string) {
  if (!id) {
    return undefined
  }

  return pakData.value.find((pak) => pak.id === id)
}

function findPakEntryByHash(header: PakHeaderInfo, hashLower: number, hashUpper: number) {
  return header.entries.find(
    (entry) =>
      (entry.hashNameLower === hashLower && entry.hashNameUpper === hashUpper) ||
      (entry.hashNameLower === hashUpper && entry.hashNameUpper === hashLower)
  )
}

function formatHex32(value: number) {
  return `0x${(value >>> 0).toString(16).toUpperCase().padStart(8, '0')}`
}

function formatHex64(hashLower: number, hashUpper: number) {
  return `0x${(hashUpper >>> 0).toString(16).toUpperCase().padStart(8, '0')}${(hashLower >>> 0)
    .toString(16)
    .toUpperCase()
    .padStart(8, '0')}`
}

function formatPropertyValue(value: unknown) {
  if (value === undefined || value === null || value === '') {
    return '—'
  }

  if (Array.isArray(value)) {
    return value.join(', ')
  }

  if (typeof value === 'object') {
    try {
      return JSON.stringify(value)
    } catch {
      return String(value)
    }
  }

  return String(value)
}

function getPakFileName(path: string) {
  const segments = splitNormalizedPath(path)
  return segments[segments.length - 1] ?? path
}

function buildDirectoryPropertySections(node: ExplorerEntry): PropertySection[] {
  const counts = getExplorerDirectoryCounts(node)

  return [
    {
      key: 'directory-basic',
      title: 'Basic Info',
      rows: [
        { key: 'directory-name', label: 'Name', value: node.name },
        { key: 'directory-path', label: 'Path', value: normalizeDisplayPath(node.path) },
        { key: 'directory-folders', label: 'Folder Count', value: String(counts.folders) },
        { key: 'directory-files', label: 'File Count', value: String(counts.files) }
      ]
    }
  ]
}

function buildFilePropertySections(
  node: ExplorerEntry,
  pak: PakInfo | undefined,
  entry: PakEntry | undefined
): PropertySection[] {
  const hashLower = node.hash?.[0]
  const hashUpper = node.hash?.[1]

  return [
    {
      key: 'file-basic',
      title: 'Basic Info',
      rows: [
        { key: 'file-name', label: 'Name', value: node.name },
        { key: 'file-path', label: 'Path', value: normalizeDisplayPath(node.path) },
        { key: 'file-source', label: 'Source', value: getExplorerSourceLabel(node.belongsTo) },
        {
          key: 'file-compressed-size',
          label: 'Compressed Size',
          value: String(node.compressedSize)
        },
        {
          key: 'file-uncompressed-size',
          label: 'Uncompressed Size',
          value: String(node.uncompressedSize)
        },
        {
          key: 'file-compressed',
          label: 'Compressed',
          value: node.isCompressed ? 'Yes' : 'No'
        }
      ]
    },
    {
      key: 'file-hash',
      title: 'Hash',
      rows: [
        {
          key: 'hash-lower',
          label: 'Hash Lower',
          value: hashLower === undefined ? '—' : formatHex32(hashLower)
        },
        {
          key: 'hash-upper',
          label: 'Hash Upper',
          value: hashUpper === undefined ? '—' : formatHex32(hashUpper)
        },
        {
          key: 'hash-mixed',
          label: 'Hash Mixed',
          value:
            hashLower === undefined || hashUpper === undefined
              ? '—'
              : formatHex64(hashLower, hashUpper)
        }
      ]
    },
    {
      key: 'file-entry',
      title: 'Original Entry',
      rows: [
        { key: 'entry-pak', label: 'Pak File', value: pak ? getPakFileName(pak.path) : '—' },
        {
          key: 'entry-offset',
          label: 'Offset',
          value: entry ? formatPropertyValue(entry.offset) : 'Matching Entry Not Found'
        },
        {
          key: 'entry-compressed',
          label: 'Compressed Size',
          value: entry ? String(entry.compressedSize) : '—'
        },
        {
          key: 'entry-uncompressed',
          label: 'Uncompressed Size',
          value: entry ? String(entry.uncompressedSize) : '—'
        },
        {
          key: 'entry-compression-type',
          label: 'Compression Type',
          value: entry ? String(entry.compressionType) : '—'
        },
        {
          key: 'entry-encryption-type',
          label: 'Encryption Type',
          value: entry ? entry.encryptionType : '—'
        },
        {
          key: 'entry-checksum',
          label: 'Checksum',
          value: entry ? entry.checksum : '—'
        },
        {
          key: 'entry-unk-attr',
          label: 'Unk Attr',
          value: entry ? entry.unkAttr : '—'
        }
      ]
    }
  ]
}

function buildPakPropertySections(pak: PakInfo, header: PakHeaderInfo): PropertySection[] {
  return [
    {
      key: 'pak-basic',
      title: 'Basic Info',
      rows: [
        { key: 'pak-file-name', label: 'File Name', value: getPakFileName(pak.path) },
        { key: 'pak-path', label: 'Path', value: pak.path },
        { key: 'pak-id', label: 'Pak ID', value: pak.id }
      ]
    },
    {
      key: 'pak-header',
      title: 'Pak Header',
      rows: [
        { key: 'pak-magic', label: 'Magic', value: formatPakMagic(header.header.magic) },
        {
          key: 'pak-major-version',
          label: 'Major Version',
          value: formatPropertyValue(header.header.majorVersion)
        },
        {
          key: 'pak-minor-version',
          label: 'Minor Version',
          value: formatPropertyValue(header.header.minorVersion)
        },
        { key: 'pak-feature', label: 'Feature', value: formatPropertyValue(header.header.feature) },
        {
          key: 'pak-total-files',
          label: 'Total Files',
          value: formatPropertyValue(header.header.totalFiles)
        },
        { key: 'pak-hash', label: 'Hash', value: formatPropertyValue(header.header.hash) },
        {
          key: 'pak-unk-sig',
          label: 'Unk U32 Sig',
          value: formatPropertyValue(header.header.unkU32Sig)
        },
        { key: 'pak-entry-count', label: 'Entry Count', value: String(header.entries.length) }
      ]
    }
  ]
}

function formatPakMagic(value: unknown) {
  if (typeof value === 'string') {
    return value
  }

  if (Array.isArray(value) && value.every((item) => typeof item === 'number')) {
    return String.fromCharCode(...value)
  }

  if (value && typeof value === 'object' && 'length' in value) {
    const list = Array.from(value as ArrayLike<unknown>)
    if (list.every((item) => typeof item === 'number')) {
      return String.fromCharCode(...(list as number[]))
    }
  }

  return formatPropertyValue(value)
}

function buildErrorPropertySections(error: unknown): PropertySection[] {
  return [
    {
      key: 'property-error',
      title: 'Read Failed',
      rows: [
        {
          key: 'property-error-message',
          label: 'Error',
          value: error instanceof Error ? error.message : String(error)
        }
      ]
    }
  ]
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

function collectFilesFromEntries(
  entries: ExplorerEntry[],
  mode: ExtractMode = 'relativePath',
  predicate?: (node: ExplorerEntry) => boolean
): ExtractFileInfo[] {
  const files = new Map<string, ExtractFileInfo>()

  const walk = (entry: ExplorerEntry, relativeRoot?: string) => {
    if (entry.isDir) {
      entry.children.forEach((child) => walk(child, relativeRoot))
      return
    }

    if (!entry.hash || !entry.belongsTo) {
      return
    }

    if (predicate && !predicate(entry)) {
      return
    }

    files.set(entry.id, {
      hash: entry.hash,
      belongsTo: entry.belongsTo,
      relativeRoot
    })
  }

  for (const entry of entries) {
    walk(entry, mode === 'relativePath' ? getSelectedItemRelativeRoot(entry.path) : undefined)
  }

  return [...files.values()]
}

function collectExtractFilesFromEntries(
  entries: ExplorerEntry[],
  mode: ExtractMode = 'absolutePath'
) {
  return collectFilesFromEntries(entries, mode)
}

function collectTextureFilesFromEntries(
  entries: ExplorerEntry[],
  mode: ExtractMode = 'relativePath'
) {
  return collectFilesFromEntries(entries, mode, (entry) => isTextureEntry(entry))
}

function collectExtractFilesFromEntry(
  entry: ExplorerEntry,
  mode: ExtractMode = 'relativePath'
): ExtractFileInfo[] {
  return collectFilesFromEntries([entry], mode)
}

function collectTextureFilesFromEntry(
  entry: ExplorerEntry,
  mode: ExtractMode = 'relativePath'
): ExtractFileInfo[] {
  return collectFilesFromEntries([entry], mode, (node) => isTextureEntry(node))
}

function bringEntryIntoTreeView(entry: ExplorerEntry) {
  if (entry.isDir) {
    openDirectory(entry.id)
    treeFocusKey.value = entry.id
    fileTreeComponent.value?.bringNodeIntoView(entry.id)
    return
  }

  setExplorerFocus(entry.id)

  if (entry.parentId) {
    currentDirectoryKey.value = entry.parentId
    treeFocusKey.value = entry.parentId
    fileTreeComponent.value?.bringNodeIntoView(entry.parentId)
  }
}

async function copyText(text: string) {
  await copyPaths([text])
}

async function copyPaths(paths: string[]) {
  const normalizedPaths = paths.map(normalizePathForClipboard)
  const text = normalizedPaths.join('\n')

  try {
    await navigator.clipboard.writeText(text)
    ShowInfo(
      normalizedPaths.length === 1
        ? t('unpack.copiedPath', { path: normalizedPaths[0] })
        : t('unpack.copiedPaths', { count: normalizedPaths.length })
    )
  } catch (error) {
    ShowError(error)
  }
}

function normalizePathForClipboard(path: string) {
  return normalizeDisplayPath(path)
}

async function openPropertiesDialog(target: ExplorerEntry | PakInfo) {
  propertiesDialogOpen.value = true
  propertiesDialogLoading.value = false
  propertiesDialogSections.value = []

  if ('children' in target) {
    if (target.isDir) {
      propertyTarget.value = { kind: 'directory', node: target }
      propertiesDialogTitle.value = `Directory Properties · ${target.name}`
      propertiesDialogDescription.value = normalizeDisplayPath(target.path)
      propertiesDialogSections.value = buildDirectoryPropertySections(target)
      return
    }

    propertyTarget.value = { kind: 'file', node: target }
    propertiesDialogTitle.value = `File Properties · ${target.name}`
    propertiesDialogDescription.value = normalizeDisplayPath(target.path)
    propertiesDialogLoading.value = true

    try {
      const pak = findPakInfo(target.belongsTo)
      const entry =
        pak && target.hash
          ? findPakEntryByHash(await getPakHeaderInfo(pak.path), target.hash[0], target.hash[1])
          : undefined
      propertiesDialogSections.value = buildFilePropertySections(target, pak, entry)
    } catch (error) {
      propertiesDialogSections.value = buildErrorPropertySections(error)
    } finally {
      propertiesDialogLoading.value = false
    }

    return
  }

  propertyTarget.value = { kind: 'pak', pak: target }
  propertiesDialogTitle.value = `Pak Properties · ${getPakFileName(target.path)}`
  propertiesDialogDescription.value = normalizeDisplayPath(target.path)
  propertiesDialogLoading.value = true

  try {
    const header = await getPakHeaderInfo(target.path)
    propertiesDialogSections.value = buildPakPropertySections(target, header)
  } catch (error) {
    propertiesDialogSections.value = buildErrorPropertySections(error)
  } finally {
    propertiesDialogLoading.value = false
  }
}

function handlePakShowProperties(pak: Pick<PakInfo, 'id' | 'path'>) {
  const target = pakData.value.find((item) => item.id === pak.id)
  if (!target) {
    return
  }

  void openPropertiesDialog(target)
}

onMounted(async () => {
  await startListenToDrop()
  await loadWorkRecords()
})

onUnmounted(async () => {
  await stopListenToDrop()
})
</script>
