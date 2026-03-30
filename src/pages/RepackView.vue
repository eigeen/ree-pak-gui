<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { exists, stat } from '@tauri-apps/plugin-fs'
import {
  CheckCircle2,
  FileArchive,
  Folder,
  FolderOpen,
  FolderPlus,
  PackagePlus,
  Square,
  Trash2
} from 'lucide-vue-next'
import FileConflict from '@/components/FileConflict.vue'
import HoverBubble from '@/components/HoverBubble.vue'
import SystemLogPanel from '@/components/SystemLogPanel.vue'
import type { MenuGroup } from '@/components/DesktopMenuBar.vue'
import PageToolbar from '@/components/PageToolbar.vue'
import { useWorkStore, type FileItem } from '@/store/work'
import { Packer, type ConflictFile, type ExportResult, type PackProgress } from '@/lib/packer'
import type { PackedPak } from '@/api/tauri/pak'
import { ensureTaskProgressIdle } from '@/service/taskProgress'
import { ShowError } from '@/utils/message'
import { useI18n } from 'vue-i18n'
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
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { Switch } from '@/components/ui/switch'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

const { t } = useI18n()
const workStore = useWorkStore()
const EXPORT_RESULT_TREE_HEIGHT = 384

type PackState = {
  exportConfig: {
    mode: 'individual' | 'single'
    autoDetectRoot: boolean
    exportDirectory: string
    fastMode: boolean
  }
  inputFiles: FileItem[]
}

type ExportTreeNode = {
  id: string
  label: string
  type: 'pak' | 'directory' | 'file'
  path: string
  sizeText?: string
  children?: ExportTreeNode[]
}

const packState = computed({
  get: () => workStore.pack as unknown as PackState,
  set: (value: PackState) => {
    ;(workStore as any).pack = value
  }
})

const packer = new Packer(
  (p: PackProgress) => {
    if (p.totalFileCount === p.finishFileCount) {
      p.currentFile = ''
    }
    progress.value = p
  },
  (result: ExportResult) => {
    exportResult.value = result
  }
)

const progress = ref<PackProgress>({
  working: false,
  currentFile: '',
  totalFileCount: 0,
  finishFileCount: 0
})

const progressValue = computed(() => {
  if (progress.value.totalFileCount === 0) return 0
  return (progress.value.finishFileCount / progress.value.totalFileCount) * 100
})

const exportResult = ref<ExportResult>({
  success: false,
  files: [],
  error: ''
})
const resultDialogVisible = ref(false)

const conflictDialogVisible = ref(false)
const conflictFiles = ref<ConflictFile[]>([])

const enableExport = computed(() => {
  if (packState.value.inputFiles.length === 0) return false
  if (
    packState.value.exportConfig.mode === 'single' &&
    !packState.value.exportConfig.exportDirectory
  ) {
    return false
  }
  return true
})
const desktopMenuItems = computed<MenuGroup[]>(() => [
  {
    key: 'resources',
    label: t('menu.resources'),
    items: [
      {
        key: 'add-folder',
        label: t('pack.addFolder'),
        icon: FolderPlus,
        action: () => handleAddViaDialog(false)
      },
      {
        key: 'add-pak',
        label: t('pack.addPak'),
        icon: PackagePlus,
        action: () => handleAddViaDialog(true)
      },
      {
        key: 'clear-files',
        label: t('pack.removeAll'),
        icon: Trash2,
        action: handleCloseAll
      }
    ]
  },
  {
    key: 'actions',
    label: t('menu.actions'),
    items: [
      {
        key: 'select-export-directory',
        label: t('pack.exportDirectory'),
        icon: FolderOpen,
        action: handleSelectDirectory
      }
    ]
  }
])

const statusText = computed(() => {
  if (progress.value.working) return t('pack.exporting')
  if (exportResult.value.success) return t('pack.exportSuccess')
  if (exportResult.value.error) return t('pack.exportFailed')
  return t('unpack.idle')
})

const exportModeLabel = computed(() =>
  packState.value.exportConfig.mode === 'single'
    ? t('pack.exportModeSingle')
    : t('pack.exportModeIndividual')
)

const exportFileCount = computed(() =>
  exportResult.value.files.reduce((count, pak) => count + pak.files.length, 0)
)

const exportTreeData = computed(() => buildExportTree(exportResult.value.files))

const exportTreeExpandedKeys = computed(() =>
  exportFileCount.value < 100 ? collectExpandableNodeIds(exportTreeData.value) : []
)

const exportTreeProps = {
  value: 'id',
  label: 'label',
  children: 'children'
}

const addFiles = async (paths: string[]) => {
  try {
    const addList: FileItem[] = []

    for (const path of paths) {
      if (!(await exists(path))) {
        ShowError(t('pack.inputFileNotFound', { path }))
        continue
      }

      if (packState.value.inputFiles.some((file: FileItem) => file.path === path)) {
        continue
      }

      const fileStat = await stat(path)
      addList.push({
        path,
        isFile: fileStat.isFile
      })
    }

    packState.value.inputFiles.push(...addList)

    if (packState.value.exportConfig.fastMode) {
      await handleExport()
      handleCloseAll()
    }
  } catch (error) {
    ShowError(error)
  }
}

const handleAddViaDialog = async (pak: boolean) => {
    const results = await openDialog({
      multiple: true,
      directory: !pak,
      filters: pak ? [{ name: t('pack.pakFilesFilter'), extensions: ['pak'] }] : undefined
    })

  if (!results) return
  await addFiles(Array.isArray(results) ? results : [results])
}

const handleCloseAll = () => {
  packState.value.inputFiles = []
}

const handleSelectDirectory = async () => {
  const result = await openDialog({
    directory: true
  })

  if (!result) return
  packState.value.exportConfig.exportDirectory = result
}

const handleRemoveFile = (index: number) => {
  packState.value.inputFiles.splice(index, 1)
}

const handleExport = async () => {
  if (!ensureTaskProgressIdle(t('global.taskBusy'))) {
    return
  }

  await packer.handleExport(packState.value.inputFiles, packState.value.exportConfig)
}

const handleTerminateExport = async () => {
  await packer.terminateExport()
}

const handleResetExport = () => {
  packer.resetExport()
}

const handleConflictResolve = () => {
  const resolutions: Record<string, number> = {}
  conflictFiles.value.forEach((conflict) => {
    resolutions[conflict.relativePath] = conflict.selectedSource
  })

  packer.setConflictResolutions(resolutions)
  conflictDialogVisible.value = false
  packer.proceedWithMergeExport(packState.value.inputFiles, packState.value.exportConfig)
}

const handleConflictCancel = () => {
  conflictDialogVisible.value = false
  handleResetExport()
}

watch(
  () => exportResult.value.success,
  (success) => {
    if (success) {
      resultDialogVisible.value = true
    }
  }
)

let unlisten: UnlistenFn | undefined

const startListenToDrop = async () => {
  unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type === 'drop') {
      await addFiles(event.payload.paths)
    }
  })
}

const stopListenToDrop = () => {
  unlisten?.()
}

onMounted(async () => {
  await startListenToDrop()
})

onUnmounted(() => {
  stopListenToDrop()
})

function buildExportTree(paks: PackedPak[]): ExportTreeNode[] {
  return paks.map((pak, pakIndex) => {
    const pakName = pak.path.split(/[/\\]/).pop() || pak.path
    const rootNode: ExportTreeNode = {
      id: `pak:${pakIndex}:${pak.path}`,
      label: `${pakName} (${pak.files.length} ${t('pack.filesCount')})`,
      type: 'pak',
      path: pak.path,
      children: []
    }

    const directoryMap = new Map<string, ExportTreeNode>([['', rootNode]])

    for (const file of pak.files) {
      const normalizedPath = file.path.replace(/\\/g, '/').replace(/^\/+/, '')
      const parts = normalizedPath.split('/').filter(Boolean)

      if (parts.length === 0) continue

      let currentPath = ''
      let parentNode = rootNode

      for (let i = 0; i < parts.length - 1; i += 1) {
        const part = parts[i]
        if (!part) continue
        currentPath = currentPath ? `${currentPath}/${part}` : part

        let directoryNode = directoryMap.get(currentPath)
        if (!directoryNode) {
          directoryNode = {
            id: `dir:${pakIndex}:${currentPath}`,
            label: part,
            type: 'directory',
            path: currentPath,
            children: []
          }
          parentNode.children ??= []
          parentNode.children.push(directoryNode)
          directoryMap.set(currentPath, directoryNode)
        }

        parentNode = directoryNode
      }

      const fileName = parts[parts.length - 1]
      if (!fileName) continue

      parentNode.children ??= []
      parentNode.children.push({
        id: `file:${pakIndex}:${normalizedPath}`,
        label: fileName,
        type: 'file',
        path: normalizedPath,
        sizeText: formatFileSize(file.size)
      })
    }

    sortExportTree(rootNode.children ?? [])
    return rootNode
  })
}

function sortExportTree(nodes: ExportTreeNode[]) {
  nodes.sort((a, b) => {
    if (a.type === 'file' && b.type !== 'file') return 1
    if (a.type !== 'file' && b.type === 'file') return -1
    return a.label.localeCompare(b.label)
  })

  for (const node of nodes) {
    if (node.children) {
      sortExportTree(node.children)
    }
  }
}

function collectExpandableNodeIds(nodes: ExportTreeNode[]): string[] {
  const ids: string[] = []

  const walk = (items: ExportTreeNode[]) => {
    for (const item of items) {
      if (item.children?.length) {
        ids.push(item.id)
        walk(item.children)
      }
    }
  }

  walk(nodes)
  return ids
}

function formatFileSize(bytes: number) {
  if (bytes === 0) return '0 B'

  const units = ['B', 'KB', 'MB', 'GB']
  let index = 0
  let current = bytes

  while (current >= 1024 && index < units.length - 1) {
    current /= 1024
    index += 1
  }

  return `${current.toFixed(current >= 10 || index === 0 ? 0 : 2)} ${units[index]}`
}
</script>

<template>
  <section class="desktop-page">
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      <PageToolbar :items="desktopMenuItems" />

      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel :default-size="26" :max-size="38" :min-size="20">
          <aside class="surface-sidebar flex h-full min-w-0 flex-col">
            <div class="editor-scrollbar flex min-h-0 flex-1 flex-col gap-4 overflow-auto p-3">
              <section class="space-y-3">
                <div>
                  <p class="section-eyebrow">{{ t('pack.exportSettings') }}</p>
                  <h2 class="section-title">{{ t('menu.repack') }}</h2>
                </div>

                <RadioGroup
                  v-model="packState.exportConfig.mode"
                  class="flex flex-col gap-0.5 rounded-lg p-1 transition-colors hover:bg-[color-mix(in_oklch,var(--surface-toolbar)_76%,var(--surface-panel))]"
                >
                  <label
                    :class="
                      packState.exportConfig.mode === 'individual'
                        ? 'flex min-h-10 items-center gap-2.5 rounded-md px-2 py-1.5 transition-colors bg-[color-mix(in_oklch,var(--color-primary)_10%,transparent)] hover:bg-[color-mix(in_oklch,var(--color-primary)_14%,transparent)]'
                        : 'flex min-h-10 items-center gap-2.5 rounded-md px-2 py-1.5 transition-colors hover:bg-[color-mix(in_oklch,var(--surface-toolbar)_52%,transparent)]'
                    "
                    for="repack-mode-individual"
                  >
                    <RadioGroupItem id="repack-mode-individual" value="individual" class="mt-0.5" />
                    <div class="min-w-0 flex-1">
                      <p class="text-sm font-medium text-foreground">
                        {{ t('pack.exportModeIndividual') }}
                      </p>
                    </div>
                  </label>
                  <label
                    :class="
                      packState.exportConfig.mode === 'single'
                        ? 'flex min-h-10 items-center gap-2.5 rounded-md px-2 py-1.5 transition-colors bg-[color-mix(in_oklch,var(--color-primary)_10%,transparent)] hover:bg-[color-mix(in_oklch,var(--color-primary)_14%,transparent)]'
                        : 'flex min-h-10 items-center gap-2.5 rounded-md px-2 py-1.5 transition-colors hover:bg-[color-mix(in_oklch,var(--surface-toolbar)_52%,transparent)]'
                    "
                    for="repack-mode-single"
                  >
                    <RadioGroupItem id="repack-mode-single" value="single" class="mt-0.5" />
                    <div class="min-w-0 flex-1">
                      <p class="text-sm font-medium text-foreground">
                        {{ t('pack.exportModeSingle') }}
                      </p>
                    </div>
                  </label>
                </RadioGroup>

                <div class="flex flex-col gap-0.5">
                  <label
                    class="flex min-h-12 items-center gap-3 rounded-lg px-3 py-2.5 transition-colors hover:bg-[color-mix(in_oklch,var(--surface-toolbar)_76%,var(--surface-panel))]"
                  >
                    <div class="min-w-0 flex-1 space-y-1">
                      <div class="flex items-center gap-2">
                        <p class="text-sm font-medium text-foreground">
                          {{ t('pack.autoDetectRoot') }}
                        </p>
                        <HoverBubble>{{ t('pack.autoDetectRootTooltip') }}</HoverBubble>
                      </div>
                    </div>
                    <Switch v-model="packState.exportConfig.autoDetectRoot" />
                  </label>

                  <label
                    class="flex min-h-12 items-center gap-3 rounded-lg px-3 py-2.5 transition-colors hover:bg-[color-mix(in_oklch,var(--surface-toolbar)_76%,var(--surface-panel))]"
                  >
                    <div class="min-w-0 flex-1 space-y-1">
                      <div class="flex items-center gap-2">
                        <p class="text-sm font-medium text-foreground">
                          {{ t('pack.fastMode') }}
                        </p>
                        <HoverBubble>
                          {{ t('pack.fastModeTooltipL1') }}<br />
                          {{ t('pack.fastModeTooltipL2') }}
                        </HoverBubble>
                      </div>
                    </div>
                    <Switch v-model="packState.exportConfig.fastMode" />
                  </label>
                </div>

                <div class="space-y-2">
                  <p class="section-eyebrow">{{ t('pack.exportDirectory') }}</p>
                  <div
                    class="flex min-h-12 items-center gap-3 rounded-lg px-3 py-2.5 transition-colors hover:bg-[color-mix(in_oklch,var(--surface-toolbar)_76%,var(--surface-panel))]"
                  >
                    <div class="min-w-0 flex-1">
                      <DenseInput
                        v-model="packState.exportConfig.exportDirectory"
                        :placeholder="t('pack.exportDirectoryPlaceholder')"
                      />
                    </div>
                    <Button
                      size="icon-sm"
                      variant="ghost"
                      class="desktop-icon-button shrink-0"
                      @click="handleSelectDirectory"
                    >
                      <FolderOpen class="size-4" />
                    </Button>
                  </div>
                </div>

                <Button
                  v-if="!progress.working"
                  :disabled="!enableExport"
                  class="w-full"
                  @click="handleExport"
                >
                  <PackagePlus class="size-4" />
                  {{ t('pack.export') }}
                </Button>

                <Button v-else variant="destructive" class="w-full" @click="handleTerminateExport">
                  <Square class="size-4" />
                  {{ t('pack.cancelExport') }}
                </Button>
              </section>
            </div>
          </aside>
        </ResizablePanel>

        <ResizableHandle class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary" />

        <ResizablePanel :default-size="74" :min-size="48">
          <ResizablePanelGroup direction="vertical">
            <ResizablePanel :default-size="72" :min-size="44">
              <div class="surface-panel flex h-full min-w-0 flex-col">
                <div class="desktop-toolbar h-10 justify-between px-3">
                  <div>
                    <h3 class="section-title">{{ t('pack.inputFilesTitle') }}</h3>
                  </div>
                  <div class="flex items-center gap-1">
                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger as-child>
                          <Button
                            size="icon-sm"
                            variant="ghost"
                            class="desktop-icon-button"
                            @click="handleAddViaDialog(false)"
                          >
                            <FolderPlus class="size-4" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent class="text-sm">{{ t('pack.addFolder') }}</TooltipContent>
                      </Tooltip>
                    </TooltipProvider>

                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger as-child>
                          <Button
                            size="icon-sm"
                            variant="ghost"
                            class="desktop-icon-button"
                            @click="handleAddViaDialog(true)"
                          >
                            <PackagePlus class="size-4" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent class="text-sm">{{ t('pack.addPak') }}</TooltipContent>
                      </Tooltip>
                    </TooltipProvider>

                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger as-child>
                          <Button
                            size="icon-sm"
                            variant="ghost"
                            class="desktop-icon-button"
                            :disabled="packState.inputFiles.length === 0"
                            @click="handleCloseAll"
                          >
                            <Trash2 class="size-4" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent class="text-sm">{{ t('pack.removeAll') }}</TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  </div>
                </div>

                <div class="editor-scrollbar min-h-0 flex-1 overflow-auto p-3">
                  <div
                    v-if="packState.inputFiles.length === 0"
                    class="empty-state h-full border-0 bg-transparent"
                  >
                    <PackagePlus class="size-8 text-muted-foreground" />
                    <p class="text-sm font-medium text-foreground">
                      {{ t('pack.noFilesAdded') }}
                    </p>
                    <p class="section-copy">{{ t('pack.noFilesAddedDesc') }}</p>
                  </div>

                  <div v-else class="flex flex-col">
                    <div
                      class="grid grid-cols-[minmax(0,1.5fr)_minmax(88px,0.45fr)_minmax(0,1.8fr)_52px] items-center gap-3 border-b border-border/80 px-3 pb-2 text-[0.6875rem] text-muted-foreground"
                    >
                      <span class="truncate">{{ t('unpack.columnName') }}</span>
                      <span class="truncate">{{ t('unpack.columnType') }}</span>
                      <span class="truncate">{{ t('pack.fileList') }}</span>
                      <span class="truncate text-right">{{ t('menu.actions') }}</span>
                    </div>

                    <div
                      v-for="(file, index) in packState.inputFiles"
                      :key="`${file.path}-${index}-main`"
                      class="grid grid-cols-[minmax(0,1.5fr)_minmax(88px,0.45fr)_minmax(0,1.8fr)_52px] items-center gap-3 border-b border-border/45 px-3 py-2 transition-colors duration-150 hover:bg-[color-mix(in_oklch,var(--surface-toolbar)_58%,transparent)]"
                    >
                      <div class="flex min-w-0 items-center gap-2.5">
                        <FileArchive
                          v-if="file.isFile"
                          class="size-4 shrink-0 text-[#f6b24f] [filter:drop-shadow(0_4px_8px_rgb(0_0_0_/_0.2))]"
                        />
                        <Folder
                          v-else
                          class="size-4 shrink-0 text-[#5db7ff] [filter:drop-shadow(0_4px_8px_rgb(0_0_0_/_0.2))]"
                        />
                        <span class="truncate text-sm font-medium text-foreground">
                          {{ file.path.split(/[\\/]/).pop() || file.path }}
                        </span>
                      </div>

                      <span class="truncate text-2xs text-muted-foreground">
                        {{ file.isFile ? t('pack.fileTypePak') : t('pack.fileTypeDirectory') }}
                      </span>

                      <span class="truncate text-2xs text-muted-foreground">
                        {{ file.path }}
                      </span>

                      <div class="flex justify-end">
                        <Button
                          size="icon-sm"
                          variant="ghost"
                          class="desktop-icon-button"
                          @click="handleRemoveFile(index)"
                        >
                          <Trash2 class="size-4" />
                        </Button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </ResizablePanel>

            <ResizableHandle class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary" />

            <ResizablePanel :default-size="28" :max-size="42" :min-size="16">
              <SystemLogPanel empty-text="No system logs yet" />
            </ResizablePanel>
          </ResizablePanelGroup>
        </ResizablePanel>
      </ResizablePanelGroup>

      <div class="desktop-statusbar">
        <div class="flex items-center gap-4">
          <span>{{ statusText }}</span>
          <span>{{ t('pack.itemsCount', { count: packState.inputFiles.length }) }}</span>
          <span>{{ exportModeLabel }}</span>
        </div>
        <div class="min-w-0 truncate text-right">
          <span>{{ packState.exportConfig.exportDirectory || t('pack.exportDirectoryUnset') }}</span>
        </div>
      </div>
    </div>

    <Dialog v-model:open="conflictDialogVisible">
      <DialogContent class="max-w-4xl rounded-[1rem] border-border/80 bg-background/96">
        <DialogHeader>
          <DialogTitle>{{ t('pack.fileConflictTitle') }}</DialogTitle>
          <DialogDescription>{{ t('pack.conflictDescription') }}</DialogDescription>
        </DialogHeader>

        <FileConflict v-model:conflicts="conflictFiles" />

        <DialogFooter>
          <Button variant="outline" @click="handleConflictCancel">{{ t('pack.cancel') }}</Button>
          <Button @click="handleConflictResolve">{{ t('pack.confirm') }}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <Dialog v-model:open="resultDialogVisible">
      <DialogContent class="max-w-4xl rounded-[1rem] border-border/80 bg-background/96">
        <DialogHeader>
          <DialogTitle>{{ t('pack.exportSuccess') }}</DialogTitle>
          <DialogDescription>{{ t('pack.exportResultDescription', { count: exportFileCount }) }}</DialogDescription>
        </DialogHeader>

        <div
          class="surface-console-panel h-[384px] overflow-hidden rounded-[0.8rem] border border-border/70 p-3"
        >
          <el-tree-v2
            :data="exportTreeData"
            :height="EXPORT_RESULT_TREE_HEIGHT"
            :props="exportTreeProps"
            node-key="id"
            :default-expanded-keys="exportTreeExpandedKeys"
            :expand-on-click-node="false"
            class="desktop-tree rounded-[0.7rem] bg-transparent"
          >
            <template #default="{ data }">
              <div class="flex min-w-0 flex-1 items-center gap-2 py-1">
                <CheckCircle2 v-if="data.type === 'pak'" class="size-4 shrink-0 text-primary" />
                <Folder
                  v-else-if="data.type === 'directory'"
                  class="size-4 shrink-0 text-sky-400"
                />
                <FileArchive v-else class="size-4 shrink-0 text-muted-foreground" />
                <span class="min-w-0 truncate text-sm text-foreground">{{ data.label }}</span>
                <span v-if="data.sizeText" class="shrink-0 text-2xs text-muted-foreground">
                  {{ data.sizeText }}
                </span>
              </div>
            </template>
          </el-tree-v2>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="resultDialogVisible = false">
            {{ t('unpack.close') }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </section>
</template>
