<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { exists, stat } from '@tauri-apps/plugin-fs'
import {
  CheckCircle2,
  CircleAlert,
  FileArchive,
  Folder,
  FolderOpen,
  FolderPlus,
  PackagePlus,
  Play,
  Square,
  Trash2
} from 'lucide-vue-next'
import FileConflict from '@/components/FileConflict.vue'
import HoverBubble from '@/components/HoverBubble.vue'
import type { MenuGroup } from '@/components/DesktopMenuBar.vue'
import PageToolbar from '@/components/PageToolbar.vue'
import { useWorkStore, type FileItem } from '@/store/work'
import { Packer, type ConflictFile, type ExportResult, type PackProgress } from '@/lib/packer'
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
import { Progress } from '@/components/ui/progress'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { Switch } from '@/components/ui/switch'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

const { t } = useI18n()
const workStore = useWorkStore()

type PackState = {
  exportConfig: {
    mode: 'individual' | 'single'
    autoDetectRoot: boolean
    exportDirectory: string
    fastMode: boolean
  }
  inputFiles: FileItem[]
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
  return 'Idle'
})

const exportModeLabel = computed(() =>
  packState.value.exportConfig.mode === 'single'
    ? t('pack.exportModeSingle')
    : t('pack.exportModeIndividual')
)

const currentStatusDetail = computed(() => {
  if (progress.value.currentFile) return progress.value.currentFile
  if (exportResult.value.error) return exportResult.value.error
  if (exportResult.value.success)
    return `${progress.value.finishFileCount} / ${progress.value.totalFileCount}`
  return packState.value.exportConfig.exportDirectory || '未设置导出目录'
})

const addFiles = async (paths: string[]) => {
  try {
    const addList: FileItem[] = []

    for (const path of paths) {
      if (!(await exists(path))) {
        ShowError(`Input file ${path} does not exist.`)
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
    filters: pak ? [{ name: 'Pak Files', extensions: ['pak'] }] : undefined
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

let unlisten: UnlistenFn | undefined

function handleToolbarExport() {
  if (!enableExport.value || progress.value.working) return
  void handleExport()
}

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
</script>

<template>
  <section class="desktop-page">
    <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
      <PageToolbar :items="desktopMenuItems" />

      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel :default-size="64" :min-size="42">
          <div class="surface-sidebar flex h-full min-w-0 flex-col">
            <div class="desktop-toolbar h-10 justify-between px-3">
              <div>
                <h2 class="section-title">{{ t('menu.repack') }}</h2>
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
                <Folder class="size-8 text-muted-foreground" />
                <p class="text-sm font-medium text-foreground">{{ t('pack.noFilesAdded') }}</p>
                <p class="section-copy">{{ t('pack.noFilesAddedDesc') }}</p>
              </div>

              <div v-else class="space-y-2">
                <div
                  v-for="(file, index) in packState.inputFiles"
                  :key="`${file.path}-${index}`"
                  class="surface-raised flex items-center gap-3 rounded-[0.7rem] border border-border/80 px-3 py-2"
                >
                  <div
                    class="flex size-8 shrink-0 items-center justify-center rounded-[0.55rem] border border-border/70 bg-background/40 text-muted-foreground"
                  >
                    <FileArchive v-if="file.isFile" class="size-4" />
                    <Folder v-else class="size-4" />
                  </div>
                  <div class="min-w-0 flex-1">
                    <p class="truncate text-sm font-medium text-foreground">{{ file.path }}</p>
                    <p class="text-ui-2xs text-muted-foreground">
                      {{ file.isFile ? 'Pak' : 'Directory' }}
                    </p>
                  </div>
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
        </ResizablePanel>

        <ResizableHandle class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary" />

        <ResizablePanel :default-size="36" :min-size="24">
          <ResizablePanelGroup direction="vertical">
            <ResizablePanel :default-size="56" :min-size="40">
              <div class="surface-panel flex h-full min-w-0 flex-col">
                <div class="desktop-toolbar h-10 justify-between px-3">
                  <h3 class="section-title">{{ t('pack.exportSettings') }}</h3>
                </div>

                <div class="editor-scrollbar min-h-0 flex-1 space-y-4 overflow-auto p-3">
                  <section class="space-y-2">
                    <p class="section-eyebrow">{{ t('pack.exportMode') }}</p>
                    <RadioGroup v-model="packState.exportConfig.mode" class="gap-2">
                      <label
                        class="surface-raised flex cursor-pointer items-start gap-3 rounded-[0.7rem] border border-border/80 px-3 py-2.5"
                        for="repack-mode-individual"
                      >
                        <RadioGroupItem
                          id="repack-mode-individual"
                          value="individual"
                          class="mt-0.5"
                        />
                        <div class="min-w-0">
                          <p class="text-ui-xs font-medium text-foreground">
                            {{ t('pack.exportModeIndividual') }}
                          </p>
                        </div>
                      </label>
                      <label
                        class="surface-raised flex cursor-pointer items-start gap-3 rounded-[0.7rem] border border-border/80 px-3 py-2.5"
                        for="repack-mode-single"
                      >
                        <RadioGroupItem id="repack-mode-single" value="single" class="mt-0.5" />
                        <div class="min-w-0">
                          <p class="text-ui-xs font-medium text-foreground">
                            {{ t('pack.exportModeSingle') }}
                          </p>
                        </div>
                      </label>
                    </RadioGroup>
                  </section>

                  <section class="space-y-2">
                    <p class="section-eyebrow">选项</p>
                    <label
                      class="surface-raised flex cursor-pointer items-start justify-between gap-4 rounded-[0.7rem] border border-border/80 px-3 py-2.5"
                    >
                      <div class="space-y-1">
                        <div class="flex items-center gap-2">
                          <p class="text-ui-xs font-medium text-foreground">
                            {{ t('pack.autoDetectRoot') }}
                          </p>
                          <HoverBubble>{{ t('pack.autoDetectRootTooltip') }}</HoverBubble>
                        </div>
                      </div>
                      <Switch v-model="packState.exportConfig.autoDetectRoot" />
                    </label>

                    <label
                      class="surface-raised flex cursor-pointer items-start justify-between gap-4 rounded-[0.7rem] border border-border/80 px-3 py-2.5"
                    >
                      <div class="space-y-1">
                        <div class="flex items-center gap-2">
                          <p class="text-ui-xs font-medium text-foreground">
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
                  </section>

                  <section class="space-y-2">
                    <p class="section-eyebrow">{{ t('pack.exportDirectory') }}</p>
                    <div class="flex items-center gap-2">
                      <DenseInput
                        v-model="packState.exportConfig.exportDirectory"
                        :placeholder="t('pack.exportDirectoryPlaceholder')"
                      />
                      <Button
                        size="icon-sm"
                        variant="outline"
                        class="desktop-icon-button shrink-0"
                        @click="handleSelectDirectory"
                      >
                        <FolderOpen class="size-4" />
                      </Button>
                    </div>
                  </section>

                  <section class="pt-1">
                    <Button
                      v-if="!progress.working"
                      :disabled="!enableExport"
                      class="w-full"
                      @click="handleExport"
                    >
                      <PackagePlus class="size-4" />
                      {{ t('pack.export') }}
                    </Button>

                    <Button
                      v-else
                      variant="destructive"
                      class="w-full"
                      @click="handleTerminateExport"
                    >
                      <Square class="size-4" />
                      {{ t('pack.cancelExport') }}
                    </Button>
                  </section>
                </div>
              </div>
            </ResizablePanel>

            <ResizableHandle class="bg-border/80 hover:bg-primary data-[dragging]:bg-primary" />

            <ResizablePanel :default-size="44" :min-size="24">
              <div class="surface-console flex h-full min-w-0 flex-col">
                <div class="desktop-toolbar h-10 justify-between px-3">
                  <div>
                    <p class="section-eyebrow">Export Status</p>
                    <h3 class="section-title">{{ statusText }}</h3>
                  </div>
                </div>

                <div class="editor-scrollbar min-h-0 flex-1 space-y-3 overflow-auto p-3">
                  <div
                    v-if="progress.working || exportResult.success || exportResult.error"
                    class="space-y-3"
                  >
                    <Progress
                      v-if="progressValue > 0"
                      :model-value="progressValue"
                      class="h-2 rounded-full"
                    />

                    <p v-if="progressValue > 0" class="text-ui-xs text-muted-foreground">
                      {{ progress.finishFileCount }} / {{ progress.totalFileCount }}
                      {{ t('pack.filesCount') }}
                    </p>

                    <div v-if="progress.currentFile" class="space-y-1">
                      <p class="section-eyebrow">{{ t('pack.exporting') }}</p>
                      <p class="break-all text-ui-xs text-foreground">{{ progress.currentFile }}</p>
                    </div>

                    <div
                      v-if="exportResult.success && !progress.working"
                      class="rounded-[0.7rem] border border-primary/25 bg-primary/8 p-3"
                    >
                      <div class="mb-2 flex items-center gap-2 text-primary">
                        <CheckCircle2 class="size-4" />
                        <span class="text-ui-xs font-medium">{{ t('pack.exportSuccess') }}</span>
                      </div>

                      <div v-if="exportResult.fileTree" class="space-y-2">
                        <p class="section-eyebrow">{{ t('pack.fileStructure') }}</p>
                        <pre
                          class="surface-console-panel editor-scrollbar max-h-56 overflow-auto rounded-[0.55rem] border border-border/70 p-3 text-ui-2xs"
                          >{{ exportResult.fileTree }}</pre
                        >
                      </div>
                    </div>

                    <div
                      v-else-if="exportResult.error && !progress.working"
                      class="rounded-[0.7rem] border border-destructive/25 bg-destructive/10 p-3"
                    >
                      <div class="mb-2 flex items-center gap-2 text-destructive">
                        <CircleAlert class="size-4" />
                        <span class="text-ui-xs font-medium">{{ t('pack.exportFailed') }}</span>
                      </div>
                      <p class="break-all text-ui-xs text-destructive">{{ exportResult.error }}</p>
                    </div>
                  </div>

                  <div v-else class="empty-state h-full border-0 bg-transparent">
                    <PackagePlus class="size-8 text-muted-foreground" />
                    <p class="text-sm font-medium text-foreground">等待导出</p>
                    <p class="section-copy">配置模式与输出目录后，即可开始打包。</p>
                  </div>
                </div>
              </div>
            </ResizablePanel>
          </ResizablePanelGroup>
        </ResizablePanel>
      </ResizablePanelGroup>

      <div class="desktop-statusbar">
        <div class="flex items-center gap-4">
          <span>{{ statusText }}</span>
          <span>{{ packState.inputFiles.length }} items</span>
          <span>{{ exportModeLabel }}</span>
        </div>
        <div class="min-w-0 truncate text-right">
          <span>{{ currentStatusDetail }}</span>
        </div>
      </div>
    </div>

    <Dialog v-model:open="conflictDialogVisible">
      <DialogContent class="max-w-4xl rounded-[1rem] border-border/80 bg-background/96">
        <DialogHeader>
          <DialogTitle>{{ t('pack.fileConflictTitle') }}</DialogTitle>
          <DialogDescription>为存在重复来源的文件选择最终保留版本。</DialogDescription>
        </DialogHeader>

        <FileConflict v-model:conflicts="conflictFiles" />

        <DialogFooter>
          <Button variant="outline" @click="handleConflictCancel">{{ t('pack.cancel') }}</Button>
          <Button @click="handleConflictResolve">{{ t('pack.confirm') }}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </section>
</template>
