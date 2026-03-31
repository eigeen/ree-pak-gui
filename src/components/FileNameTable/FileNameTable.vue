<script setup lang="ts">
import { computed, onMounted, ref, useAttrs, watch } from 'vue'
import { openPath } from '@tauri-apps/plugin-opener'
import {
  AlertTriangle,
  Cloud,
  CloudDownload,
  Download,
  FolderOpen,
  HardDrive,
  RefreshCw,
  Trash2,
  Wrench
} from 'lucide-vue-next'
import { getFileListDir } from '@/lib/localDir'
import type { FileListSource } from '@/lib/NameListFile'
import { fileListService } from '@/service/filelist'
import { useFileListStore } from '@/store/filelist'
import { ShowError, ShowInfo } from '@/utils/message'
import { getFileStem } from '@/utils/path'
import { useI18n } from 'vue-i18n'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog'
import FileNameTableSelector from '@/components/FileNameTable/FileNameTableSelector.vue'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

const { t } = useI18n()
const attrs = useAttrs()

interface Props {
  showManageButton?: boolean
  showSelector?: boolean
  showManageEntryInSelector?: boolean
}

type RemoteItemStatus =
  | 'downloadable'
  | 'latest'
  | 'updating'
  | 'downloading'
  | 'updateable'
  | 'conflict'

interface RemoteFileListItem {
  identifier: string
  fileName: string
  updateTime: Date
  status: RemoteItemStatus
}

const filelistStore = useFileListStore()
const selectedValue = defineModel<string>({ default: '' })
const props = withDefaults(defineProps<Props>(), {
  showManageButton: true,
  showSelector: true,
  showManageEntryInSelector: false
})

const showMenu = ref(false)
const fetchingRemote = ref(false)
const localListSelected = ref<string[]>([])
const hasFetchedRemote = ref(false)
const downloadableItems = ref<RemoteFileListItem[]>([])

const localSources = computed<FileListSource[]>(() => {
  const itemsMap: Record<string, FileListSource> = {}

  for (const identifier in filelistStore.localFile) {
    const localFile = filelistStore.localFile[identifier]
    if (localFile) {
      itemsMap[identifier] = { ...localFile.source }
    }
  }

  for (const fileName in filelistStore.downloadedFile) {
    const downloadedFile = filelistStore.downloadedFile[fileName]
    if (!downloadedFile) continue

    const source = downloadedFile.source
    if (!(source.identifier in itemsMap)) {
      itemsMap[source.identifier] = { ...source }
    }
  }

  return Object.values(itemsMap).sort((a, b) => a.identifier.localeCompare(b.identifier))
})

const comboItems = computed(() =>
  localSources.value.map((item) => ({
    label: item.identifier,
    value: item.identifier
  }))
)

watch(
  () => [filelistStore.localFile, filelistStore.downloadedFile, filelistStore.remoteManifest],
  () => {
    const items: RemoteFileListItem[] = []

    for (const fileName in filelistStore.remoteManifest) {
      const source = filelistStore.remoteManifest[fileName]
      if (!source) continue

      const identifier = getFileStem(source.file_name)
      const localFile =
        filelistStore.localFile[identifier] ?? filelistStore.downloadedFile[identifier]

      let status: RemoteItemStatus = 'downloadable'
      if (localFile) {
        if (localFile.source.sourceType === 'local') {
          status = 'conflict'
        } else {
          const localUpdateTime = new Date(localFile.getMetadata<string>('update_time') ?? 0)
          const remoteUpdateTime = new Date(source.update_time)
          status = remoteUpdateTime > localUpdateTime ? 'updateable' : 'latest'
        }
      }

      items.push({
        identifier,
        fileName,
        updateTime: new Date(source.update_time),
        status
      })
    }

    downloadableItems.value = items
  },
  { deep: true }
)

const getSourceMeta = (source: FileListSource) => {
  if (source.sourceType === 'local') {
    return {
      label: t('fileNameTable.sourceLocal'),
      icon: HardDrive
    }
  }

  return {
    label: t('fileNameTable.sourceRemote'),
    icon: Cloud
  }
}

const getStatusLabel = (status: RemoteItemStatus) => {
  switch (status) {
    case 'latest':
      return t('fileNameTable.statusLatest')
    case 'updateable':
      return t('fileNameTable.statusUpdate')
    case 'updating':
      return t('fileNameTable.statusUpdating')
    case 'downloading':
      return t('fileNameTable.statusDownloading')
    case 'conflict':
      return t('fileNameTable.statusConflict')
    default:
      return t('fileNameTable.statusDownload')
  }
}

async function handleUpdateItem(item: RemoteFileListItem) {
  const oldStatus = item.status
  item.status = 'updating'

  try {
    await fileListService.downloadRemoteFile(item.fileName)
    item.status = 'latest'
    await handleRefreshLocal()
  } catch (err) {
    ShowError(t('fileNameTable.failedDownloadRemote', { error: String(err) }))
    item.status = oldStatus
  }
}

async function handleDownload(item: RemoteFileListItem) {
  const oldStatus = item.status
  item.status = 'downloading'

  try {
    await fileListService.downloadRemoteFile(item.fileName)
    item.status = 'latest'
    await handleRefreshLocal()
  } catch (err) {
    ShowError(t('fileNameTable.failedDownloadRemote', { error: String(err) }))
    item.status = oldStatus
  }
}

async function handleOpenLocalDir() {
  const dir = await getFileListDir(true)
  await openPath(dir)
}

async function handleFetchRemote() {
  fetchingRemote.value = true
  try {
    await filelistStore.fetchRemoteSource()
  } catch (err) {
    ShowError(t('fileNameTable.failedFetchRemote', { error: String(err) }))
  } finally {
    fetchingRemote.value = false
  }
}

async function handleRefreshLocal() {
  await filelistStore.refreshLocalSource()
  localListSelected.value = []
}

async function handleDeleteLocal() {
  if (localListSelected.value.length === 0) {
    return
  }

  try {
    for (const identifier of localListSelected.value) {
      if (identifier in filelistStore.downloadedFile) {
        await fileListService.removeDownloaded(identifier)
      } else if (identifier in filelistStore.localFile) {
        await fileListService.removeLocal(identifier)
      }
    }
  } catch (err) {
    ShowError(err)
    return
  }

  ShowInfo(t('fileNameTable.selectedFilesDeleted'))
  await handleRefreshLocal()
}

function openManager() {
  showMenu.value = true
}

watch(showMenu, async (val) => {
  if (val && !hasFetchedRemote.value) {
    hasFetchedRemote.value = true
    await handleFetchRemote()
  }
})

onMounted(async () => {
  try {
    await filelistStore.refreshLocalSource()
  } catch (err) {
    ShowError(err)
  }
})

defineExpose({ openManager })
</script>

<template>
  <div v-bind="attrs" class="space-y-4">
    <Button
      v-if="props.showManageButton"
      class="w-full justify-center rounded-xl"
      variant="outline"
      @click="showMenu = true"
    >
      <Wrench class="size-4" />
      {{ t('fileNameTable.manageFileList') }}
    </Button>

    <FileNameTableSelector
      v-if="props.showSelector"
      v-model="selectedValue"
      :items="comboItems"
      :leading-action-label="
        props.showManageEntryInSelector ? t('fileNameTable.manageFileList') : undefined
      "
      @leading-action="openManager"
    />

    <Dialog v-model:open="showMenu">
      <DialogContent
        class="max-w-[min(1100px,calc(100vw-2rem))] rounded-[1.5rem] border-white/60 bg-background/96 p-0 sm:max-w-[1100px]"
      >
        <DialogHeader class="border-b border-border/70 px-6 py-5">
          <DialogTitle>{{ t('fileNameTable.manageFileList') }}</DialogTitle>
          <DialogDescription>{{ t('fileNameTable.managerDescription') }}</DialogDescription>
        </DialogHeader>

        <div class="flex flex-col gap-6 px-6 py-6">
          <div class="flex flex-wrap gap-3">
            <Button variant="outline" @click="handleOpenLocalDir">
              <FolderOpen class="size-4" />
              {{ t('fileNameTable.openLocalDir') }}
            </Button>
            <Button variant="outline" :disabled="fetchingRemote" @click="handleFetchRemote">
              <CloudDownload class="size-4" />
              {{ t('fileNameTable.fetchRemote') }}
            </Button>
          </div>

          <div class="grid gap-6 lg:grid-cols-[minmax(0,1.15fr)_1px_minmax(0,0.95fr)] lg:gap-0">
            <section class="flex min-h-[28rem] flex-col p-1 pr-3 lg:p-0 lg:pr-6">
              <div class="mb-4 flex items-center justify-between gap-3">
                <div>
                  <p class="section-eyebrow">{{ t('fileNameTable.local') }}</p>
                  <h3 class="text-base font-semibold">{{ t('fileNameTable.local') }}</h3>
                </div>
                <Badge variant="outline">{{ localSources.length }}</Badge>
              </div>

              <ScrollArea class="min-h-0 flex-1 pr-2">
                <div class="space-y-2">
                  <label
                    v-for="item in localSources"
                    :key="item.identifier"
                    class="flex cursor-pointer items-center gap-3 rounded-2xl border border-border/70 bg-background/85 px-3 py-3 transition hover:border-primary/30 hover:bg-accent/20"
                  >
                    <input
                      v-model="localListSelected"
                      :value="item.identifier"
                      class="size-4 rounded border-input text-primary focus:ring-ring/30"
                      type="checkbox"
                    />
                    <component :is="getSourceMeta(item).icon" class="size-4 text-muted-foreground" />
                    <div class="min-w-0 flex-1">
                      <p class="truncate text-sm font-medium">{{ item.identifier }}</p>
                      <p class="truncate text-xs text-muted-foreground">
                        {{ getSourceMeta(item).label }}
                      </p>
                    </div>
                  </label>
                </div>
              </ScrollArea>

              <div class="mt-4 flex flex-wrap gap-2 border-t border-border/70 pt-4">
                <Button variant="outline" @click="handleRefreshLocal">
                  <RefreshCw class="size-4" />
                  {{ t('fileNameTable.refresh') }}
                </Button>
                <Button
                  variant="outline"
                  :disabled="localListSelected.length === 0"
                  @click="handleDeleteLocal"
                >
                  <Trash2 class="size-4" />
                  {{ t('fileNameTable.delete') }}
                </Button>
              </div>
            </section>

            <div class="hidden bg-border/70 lg:block" aria-hidden="true" />

            <section class="flex min-h-[28rem] flex-col p-1 pl-3 lg:p-0 lg:pl-6">
              <div class="mb-4 flex items-center justify-between gap-3">
                <div>
                  <p class="section-eyebrow">{{ t('fileNameTable.downloadable') }}</p>
                  <h3 class="text-base font-semibold">{{ t('fileNameTable.downloadable') }}</h3>
                </div>
                <Badge variant="outline">{{ downloadableItems.length }}</Badge>
              </div>

              <ScrollArea class="min-h-0 flex-1 pr-2">
                <div class="space-y-2">
                  <div
                    v-for="item in downloadableItems"
                    :key="item.identifier"
                    class="flex items-center gap-3 rounded-2xl border border-border/70 bg-background/85 px-3 py-3"
                  >
                    <div class="min-w-0 flex-1">
                      <p class="truncate text-sm font-medium">{{ item.identifier }}</p>
                      <p class="truncate text-xs text-muted-foreground">
                        {{ item.updateTime.toLocaleString() }}
                      </p>
                    </div>

                    <Badge
                      variant="outline"
                      :class="
                        item.status === 'conflict'
                          ? 'border-destructive/40 bg-destructive/10 text-destructive'
                          : item.status === 'latest'
                            ? 'border-primary/25 bg-primary/10 text-primary'
                            : ''
                      "
                    >
                      {{ getStatusLabel(item.status) }}
                    </Badge>

                    <Button
                      v-if="['downloadable', 'downloading'].includes(item.status)"
                      size="icon-sm"
                      variant="outline"
                      :disabled="item.status === 'downloading'"
                      @click="handleDownload(item)"
                    >
                      <Download class="size-4" />
                    </Button>

                    <Button
                      v-else-if="['updateable', 'updating'].includes(item.status)"
                      size="icon-sm"
                      variant="outline"
                      :disabled="item.status === 'updating'"
                      @click="handleUpdateItem(item)"
                    >
                      <RefreshCw class="size-4" />
                    </Button>

                    <TooltipProvider v-else-if="item.status === 'conflict'">
                      <Tooltip>
                        <TooltipTrigger as-child>
                          <button
                            class="inline-flex size-8 items-center justify-center rounded-full border border-destructive/30 bg-destructive/10 text-destructive"
                            type="button"
                          >
                            <AlertTriangle class="size-4" />
                          </button>
                        </TooltipTrigger>
                        <TooltipContent class="max-w-72 rounded-xl px-3 py-2 text-sm leading-6">
                          <div>
                            <span>{{ t('fileNameTable.conflictDownloadTip1') }}</span>
                            <br />
                            <span>{{ t('fileNameTable.conflictDownloadTip2') }}</span>
                          </div>
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  </div>
                </div>
              </ScrollArea>
            </section>
          </div>
        </div>

        <DialogFooter class="border-t border-border/70 px-6 py-4">
          <Button variant="outline" @click="showMenu = false">{{ t('unpack.close') }}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
