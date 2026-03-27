<template>
  <section class="space-y-6">
    <div class="space-y-1">
      <p class="section-eyebrow">Unpack Workflow</p>
      <h2 class="section-title">{{ t('menu.unpack') }}</h2>
      <p class="section-copy">保留 pak 加载、文件树浏览、筛选与解包流程，仅替换外围 UI 体系。</p>
    </div>

    <div class="grid gap-6 xl:grid-cols-[20rem_minmax(0,1fr)]">
      <aside class="space-y-6">
        <section class="app-panel flex flex-col gap-4 p-5">
          <div>
            <p class="section-eyebrow">{{ t('unpack.fileList') }}</p>
            <h3 class="section-title">{{ t('unpack.fileList') }}</h3>
          </div>
          <FileNameTable v-model="unpackState.fileList" />
        </section>

        <section class="app-panel flex flex-col gap-4 p-5">
          <div>
            <p class="section-eyebrow">{{ t('unpack.pakFiles') }}</p>
            <h3 class="section-title">{{ t('unpack.pakFiles') }}</h3>
          </div>
          <PakFiles
            :enable-add="enableAddPaks"
            :pak-list="pakData"
            @close="handleClose"
            @close-all="handleCloseAll"
            @open="handleOpen"
            @order="handleOrder"
          />
        </section>

        <section class="app-panel flex flex-col gap-4 p-5">
          <div>
            <p class="section-eyebrow">Filter</p>
            <h3 class="section-title">{{ t('unpack.filterKeyword') }}</h3>
          </div>

          <div class="space-y-3">
            <Input v-model="unpackState.filterText" :placeholder="t('unpack.filterKeyword')" />

            <label
              class="flex items-center gap-3 rounded-2xl border border-border/70 bg-secondary/25 px-4 py-3"
            >
              <input v-model="unpackState.filterUseRegex" class="size-4" type="checkbox" />
              <span class="text-sm font-medium text-foreground">{{ t('unpack.regex') }}</span>
            </label>

            <Button
              variant="outline"
              :disabled="unpackState.filterText === filterTextApply"
              @click="updateFilter"
            >
              <Filter class="size-4" />
              {{ t('unpack.applyFilter') }}
            </Button>
          </div>
        </section>
      </aside>

      <div
        class="grid gap-4"
        :class="isPreviewExpanded ? '2xl:grid-cols-[minmax(0,1fr)_24rem]' : ''"
      >
        <section class="app-panel relative flex min-h-[42rem] flex-col p-3 sm:p-4">
          <div
            v-if="showOverlay"
            class="absolute inset-3 z-20 flex items-center justify-center rounded-[1.25rem] bg-background/70 backdrop-blur-sm"
            @click.stop
          >
            <Button :disabled="loadingTree" @click="doRender">
              <RefreshCw class="size-4" :class="loadingTree ? 'animate-spin' : ''" />
              {{ t('unpack.loadFileTree') }}
            </Button>
          </div>

          <div v-if="pakData.length === 0" class="empty-state flex-1">
            <FileArchive class="size-12 text-muted-foreground" />
            <p class="text-base font-semibold text-foreground">尚未添加文件</p>
            <p class="section-copy">点击左侧按钮或拖拽文件到此处添加。</p>
          </div>

          <template v-else>
            <div
              class="flex flex-1 flex-col overflow-hidden rounded-[1.15rem] border border-border/70 bg-secondary/25"
            >
              <div class="min-h-0 flex-1 overflow-hidden p-3">
                <FileTree
                  ref="fileTreeComponent"
                  :data="treeData"
                  :filter-text="filterTextApply"
                  :regex-mode="unpackState.filterUseRegex"
                  class="h-full"
                  @node-click="handleNodeClick"
                />
              </div>

              <div class="flex flex-wrap justify-end gap-3 border-t border-border/70 px-4 py-4">
                <Button :disabled="!enableExtract" @click="doExtraction">
                  <Download class="size-4" />
                  {{ t('unpack.extract') }}
                </Button>
              </div>
            </div>

            <Button
              size="icon"
              variant="outline"
              class="absolute right-3 top-3 z-10 rounded-full"
              @click="togglePreviewPane"
            >
              <ChevronRight v-if="isPreviewExpanded" class="size-4" />
              <ChevronLeft v-else class="size-4" />
            </Button>
          </template>
        </section>

        <div v-if="isPreviewExpanded" class="min-h-[42rem]">
          <PreviewPane :file-name="previewFileName" :preview-uri="previewUri" />
        </div>
      </div>
    </div>

    <Dialog v-model:open="showProgressPanel">
      <DialogContent
        class="max-w-lg rounded-[1.5rem] border-white/60 bg-background/96"
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
          <Progress :model-value="progressValue" class="h-2.5 rounded-full" />
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
      <AlertDialogContent class="rounded-[1.5rem] border-white/60 bg-background/96">
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
import { LogicalSize, ProgressBarStatus, getCurrentWindow } from '@tauri-apps/api/window'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import { exists } from '@tauri-apps/plugin-fs'
import {
  ChevronLeft,
  ChevronRight,
  Download,
  FileArchive,
  Filter,
  RefreshCw
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
import { Input } from '@/components/ui/input'
import { Progress } from '@/components/ui/progress'

const { t } = useI18n()
const workStore = useWorkStore()
type UnpackState = {
  fileList: string
  paks: string[]
  filterText: string
  filterUseRegex: boolean
}
const unpackState = computed({
  get: () => workStore.unpack as unknown as UnpackState,
  set: (value: UnpackState) => {
    ;(workStore as any).unpack = value
  }
})

const filterTextApply = ref('')
const pakData = ref<PakInfo[]>([])
const initialLoaded = ref(false)
const treeData = ref<RenderTreeNode | null>(null)
const showOverlay = ref(false)
const loadingTree = ref(false)
const isPreviewExpanded = ref(false)
const originalWindowSize = ref<{ width: number; height: number } | null>(null)
const previewUri = ref('')
const previewFileName = ref('')
const unpackWorking = ref(false)
const showProgressPanel = ref(false)
const currentFile = ref('')
const totalFileCount = ref(0)
const finishFileCount = ref(0)
const showConfirmTermination = ref(false)

const progressValue = computed(() =>
  totalFileCount.value === 0 ? 0 : (finishFileCount.value / totalFileCount.value) * 100
)

const enableAddPaks = computed(() => unpackState.value.fileList !== '')
const enableExtract = computed(() => treeData.value !== null)

const fileTreeComponent = ref<InstanceType<typeof FileTree>>()

watch(pakData, async () => {
  treeData.value = null
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

    if (!result) {
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

    if (!selected) {
      return
    }

    if (Array.isArray(selected)) {
      selected = selected[0]
    }

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
        window.setProgressBar({
          status: ProgressBarStatus.Normal,
          progress: 0
        })
      } else if (event.event === 'workFinished') {
        unpackWorking.value = false
        if (finishFileCount.value !== totalFileCount.value) {
          finishFileCount.value = totalFileCount.value
        }
        window.setProgressBar({
          status: ProgressBarStatus.None,
          progress: 0
        })
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

let unlisten: UnlistenFn | undefined

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

async function togglePreviewPane() {
  const window = getCurrentWindow()

  try {
    if (!isPreviewExpanded.value) {
      const size = await window.innerSize()
      const scale = await window.scaleFactor()
      originalWindowSize.value = { width: size.width, height: size.height }
      await window.setSize(new LogicalSize(size.width + 400 / scale, size.height / scale))
      isPreviewExpanded.value = true
    } else {
      if (originalWindowSize.value) {
        const scale = await window.scaleFactor()
        await window.setSize(
          new LogicalSize(
            originalWindowSize.value.width / scale,
            originalWindowSize.value.height / scale
          )
        )
      }
      isPreviewExpanded.value = false
    }
  } catch (error) {
    ShowError(String(error))
  }
}

function parseId(id: string): JsSafeHash {
  return id.split(',').map((str) => parseInt(str, 10)) as JsSafeHash
}

async function handleNodeClick(data: TreeData) {
  try {
    if (!isPreviewExpanded.value) {
      return
    }

    if (data.children && data.children.length > 0) {
      previewUri.value = ''
      previewFileName.value = ''
      return
    }

    const hash = parseId(data.id)
    const previewFile = await getPreviewFile(hash)
    previewUri.value = convertFileSrc(previewFile, 'asset')
    previewFileName.value = data.label
  } catch {
    previewUri.value = ''
    previewFileName.value = ''
  }
}

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

watch(isPreviewExpanded, (expanded) => {
  if (!expanded) {
    previewUri.value = ''
    previewFileName.value = ''
  }
})

async function loadWorkRecords() {
  await workStore.loadWorkRecords()
  if (initialLoaded.value) {
    return
  }

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

onMounted(async () => {
  await startListenToDrop()
  await loadWorkRecords()
})

onUnmounted(async () => {
  await stopListenToDrop()
})
</script>
