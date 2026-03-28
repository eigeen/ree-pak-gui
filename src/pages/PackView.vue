<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { exists, stat } from '@tauri-apps/plugin-fs'
import {
  CheckCircle2,
  CircleAlert,
  Folder,
  FolderOpen,
  FolderPlus,
  PackagePlus,
  Square,
  Trash2
} from 'lucide-vue-next'
import FileConflict from '@/components/FileConflict.vue'
import HoverBubble from '@/components/HoverBubble.vue'
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
import { Input } from '@/components/ui/input'
import { Progress } from '@/components/ui/progress'

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

const handleWindowAddFolder = () => {
  void handleAddViaDialog(false)
}

const handleWindowAddPak = () => {
  void handleAddViaDialog(true)
}

const handleWindowClearFiles = () => {
  handleCloseAll()
}

const handleWindowSelectExportDirectory = () => {
  void handleSelectDirectory()
}

const handleWindowExport = () => {
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
  window.addEventListener('pack:add-folder', handleWindowAddFolder)
  window.addEventListener('pack:add-pak', handleWindowAddPak)
  window.addEventListener('pack:clear-files', handleWindowClearFiles)
  window.addEventListener('pack:select-export-directory', handleWindowSelectExportDirectory)
  window.addEventListener('pack:export', handleWindowExport)
  await startListenToDrop()
})

onUnmounted(() => {
  window.removeEventListener('pack:add-folder', handleWindowAddFolder)
  window.removeEventListener('pack:add-pak', handleWindowAddPak)
  window.removeEventListener('pack:clear-files', handleWindowClearFiles)
  window.removeEventListener('pack:select-export-directory', handleWindowSelectExportDirectory)
  window.removeEventListener('pack:export', handleWindowExport)
  stopListenToDrop()
})
</script>

<template>
  <section class="space-y-6">
    <div class="space-y-1">
      <p class="section-eyebrow">Pack Workflow</p>
      <h2 class="section-title">{{ t('menu.repack') }}</h2>
      <p class="section-copy">批量收集目录后导出 pak 文件，保留原有工作流与拖拽行为。</p>
    </div>

    <div class="grid gap-6 xl:grid-cols-[minmax(0,1fr)_22rem]">
      <section class="app-panel flex min-h-[38rem] flex-col p-5">
        <div class="mb-5 flex flex-wrap items-center justify-between gap-3">
          <div>
            <p class="section-eyebrow">{{ t('pack.fileList') }}</p>
            <h3 class="section-title">{{ t('pack.fileList') }}</h3>
          </div>
          <div class="flex flex-wrap gap-2">
            <Button @click="handleAddViaDialog(false)">
              <FolderPlus class="size-4" />
              {{ t('pack.addFolder') }}
            </Button>
            <Button variant="outline" @click="handleCloseAll">
              <Trash2 class="size-4" />
              {{ t('pack.removeAll') }}
            </Button>
          </div>
        </div>

        <div class="app-panel-muted flex min-h-0 flex-1 flex-col p-3">
          <div v-if="packState.inputFiles.length === 0" class="empty-state flex-1">
            <Folder class="size-12 text-muted-foreground" />
            <p class="text-base font-semibold text-foreground">{{ t('pack.noFilesAdded') }}</p>
            <p class="section-copy">{{ t('pack.noFilesAddedDesc') }}</p>
          </div>

          <div v-else class="editor-scrollbar flex-1 space-y-2 overflow-auto pr-1">
            <div
              v-for="(file, index) in packState.inputFiles"
              :key="`${file.path}-${index}`"
              class="flex items-center gap-3 rounded-[1rem] border border-border/70 bg-background/85 px-3 py-3"
            >
              <div
                class="flex size-9 shrink-0 items-center justify-center rounded-2xl border border-primary/20 bg-primary/10 text-primary"
              >
                <Folder class="size-4" />
              </div>
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-medium text-foreground">{{ file.path }}</p>
              </div>
              <Button
                size="icon-sm"
                variant="ghost"
                class="rounded-full"
                @click="handleRemoveFile(index)"
              >
                <Trash2 class="size-4" />
              </Button>
            </div>
          </div>
        </div>
      </section>

      <section class="app-panel flex flex-col gap-5 p-5">
        <div>
          <p class="section-eyebrow">{{ t('pack.exportSettings') }}</p>
          <h3 class="section-title">{{ t('pack.exportSettings') }}</h3>
        </div>

        <div class="space-y-5">
          <div class="space-y-3">
            <p class="text-sm font-medium text-foreground">{{ t('pack.exportMode') }}</p>
            <label
              class="flex items-start gap-3 rounded-2xl border border-border/70 bg-secondary/25 px-4 py-3"
            >
              <input
                v-model="packState.exportConfig.mode"
                class="mt-1 size-4"
                type="radio"
                value="individual"
              />
              <div>
                <p class="text-sm font-medium text-foreground">
                  {{ t('pack.exportModeIndividual') }}
                </p>
              </div>
            </label>
          </div>

          <div class="space-y-3">
            <label
              class="flex items-start justify-between gap-4 rounded-2xl border border-border/70 bg-secondary/25 px-4 py-3"
            >
              <div class="space-y-1">
                <div class="flex items-center gap-2">
                  <p class="text-sm font-medium text-foreground">{{ t('pack.autoDetectRoot') }}</p>
                  <HoverBubble>{{ t('pack.autoDetectRootTooltip') }}</HoverBubble>
                </div>
              </div>
              <input
                v-model="packState.exportConfig.autoDetectRoot"
                class="mt-1 size-4"
                type="checkbox"
              />
            </label>

            <label
              class="flex items-start justify-between gap-4 rounded-2xl border border-border/70 bg-secondary/25 px-4 py-3"
            >
              <div class="space-y-1">
                <div class="flex items-center gap-2">
                  <p class="text-sm font-medium text-foreground">{{ t('pack.fastMode') }}</p>
                  <HoverBubble>
                    {{ t('pack.fastModeTooltipL1') }}<br />
                    {{ t('pack.fastModeTooltipL2') }}
                  </HoverBubble>
                </div>
              </div>
              <input
                v-model="packState.exportConfig.fastMode"
                class="mt-1 size-4"
                type="checkbox"
              />
            </label>
          </div>

          <div class="space-y-3">
            <p class="text-sm font-medium text-foreground">{{ t('pack.exportDirectory') }}</p>
            <div class="flex gap-2">
              <Input
                v-model="packState.exportConfig.exportDirectory"
                :placeholder="t('pack.exportDirectoryPlaceholder')"
              />
              <Button size="icon" variant="outline" class="shrink-0" @click="handleSelectDirectory">
                <FolderOpen class="size-4" />
              </Button>
            </div>
          </div>
        </div>

        <div class="space-y-4">
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

          <div
            v-if="progress.working || exportResult.success || exportResult.error"
            class="space-y-3"
          >
            <Progress
              v-if="progressValue > 0"
              :model-value="progressValue"
              class="h-2.5 rounded-full"
            />

            <p v-if="progressValue > 0" class="text-sm text-muted-foreground">
              {{ progress.finishFileCount }} / {{ progress.totalFileCount }}
              {{ t('pack.filesCount') }}
            </p>

            <div v-if="progress.currentFile" class="space-y-1">
              <p class="text-sm font-medium text-foreground">{{ t('pack.exporting') }}</p>
              <p class="break-all text-sm text-muted-foreground">{{ progress.currentFile }}</p>
            </div>

            <div
              v-if="exportResult.success && !progress.working"
              class="space-y-3 rounded-2xl border border-primary/20 bg-primary/8 p-4"
            >
              <div class="flex items-center gap-2 text-primary">
                <CheckCircle2 class="size-4" />
                <span class="text-sm font-medium">{{ t('pack.exportSuccess') }}</span>
              </div>

              <div v-if="exportResult.fileTree" class="space-y-2">
                <p class="text-sm font-medium text-foreground">{{ t('pack.fileStructure') }}</p>
                <pre
                  class="editor-scrollbar max-h-56 overflow-auto rounded-2xl border border-border/70 bg-background/90 p-3 text-xs leading-6"
                  >{{ exportResult.fileTree }}</pre
                >
              </div>
            </div>

            <div
              v-else-if="exportResult.error && !progress.working"
              class="rounded-2xl border border-destructive/25 bg-destructive/10 p-4"
            >
              <div class="mb-2 flex items-center gap-2 text-destructive">
                <CircleAlert class="size-4" />
                <span class="text-sm font-medium">{{ t('pack.exportFailed') }}</span>
              </div>
              <p class="break-all text-sm text-destructive">{{ exportResult.error }}</p>
            </div>
          </div>
        </div>
      </section>
    </div>

    <Dialog v-model:open="conflictDialogVisible">
      <DialogContent class="max-w-4xl rounded-[1.5rem] border-white/60 bg-background/96">
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
