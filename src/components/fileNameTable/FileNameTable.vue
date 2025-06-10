<script setup lang="ts">
import { getFileListDir } from '@/lib/localDir'
import type { FileListSource, NameListFile } from '@/lib/NameListFile'
import { FileListService } from '@/service/filelist'
import { useFileListStore } from '@/store/filelist'
import { ShowError, ShowInfo } from '@/utils/message'
import { getFileStem } from '@/utils/path'
import { openPath } from '@tauri-apps/plugin-opener'
import { computed, onMounted, ref, watch } from 'vue'

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

const selectedValue = defineModel<string>()

const showMenu = ref(false)
const leftPanelWidth = ref(6)
const rightPanelWidth = ref(4)
const fetchingRemote = ref(false)
const localListSelected = ref<string[]>([])

/**
 * Mixed sources of local and remote sources.
 * Local sources will override remote sources with the same identifier.
 */
const localSources = computed<FileListSource[]>(() => {
  const itemsMap: { [identifier: string]: FileListSource } = {}
  for (const identifier in filelistStore.localFile) {
    itemsMap[identifier] = {
      ...filelistStore.localFile[identifier].source
    }
  }
  for (const fileName in filelistStore.downloadedFile) {
    const source = filelistStore.downloadedFile[fileName].source
    const identifier = source.identifier
    if (identifier in itemsMap) {
      console.warn(
        `Duplicate identifier ${identifier} found in local and remote sources, using local source.`
      )
      continue
    }
    itemsMap[identifier] = {
      ...source
    }
  }

  // order by identifier
  const sources = Object.values(itemsMap)
  sources.sort((a, b) => a.identifier.localeCompare(b.identifier))

  return sources
})

const downloadableItems = ref<RemoteFileListItem[]>([])

// auto update downloadable items
watch(
  () => [filelistStore.localFile, filelistStore.downloadedFile, filelistStore.remoteManifest],
  () => {
    const items: RemoteFileListItem[] = []
    for (const fileName in filelistStore.remoteManifest) {
      const source = filelistStore.remoteManifest[fileName]
      const identifier = getFileStem(source.file_name)

      let status: RemoteItemStatus = 'downloadable'
      const isOnLocal =
        identifier in filelistStore.localFile || identifier in filelistStore.downloadedFile
      if (isOnLocal) {
        const localFile =
          filelistStore.localFile[identifier] || filelistStore.downloadedFile[identifier]

        // if on local manually folder, conflict
        if (localFile.source.sourceType === 'local') {
          status = 'conflict'
        } else {
          // check pub time to see if update available
          const localUpdateTime = new Date(localFile.getMetadata<string>('update_time') ?? 0)
          const remoteUpdateTime = new Date(source.update_time)
          if (remoteUpdateTime > localUpdateTime) {
            status = 'updateable'
          } else {
            status = 'latest'
          }
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

const headers = [{ title: '', key: 'identifier' }]

async function handleUpdateItem(item: RemoteFileListItem) {
  console.log('Updating item:', item.identifier)
  const oldStatus = item.status
  item.status = 'updating'

  try {
    const srv = FileListService.getInstance()
    await srv.downloadRemoteFile(item.fileName)
  } catch (err) {
    ShowError(err)
    item.status = oldStatus
  }
  console.log('Update finished:', item.identifier)
  item.status = 'latest'
  await handleRefreshLocal()
}

async function handleDownload(item: RemoteFileListItem) {
  console.log('Downloading item:', item.identifier)
  const oldStatus = item.status
  item.status = 'downloading'

  try {
    const srv = FileListService.getInstance()
    await srv.downloadRemoteFile(item.fileName)
  } catch (err) {
    ShowError(err)
    item.status = oldStatus
  }
  console.log('Download finished:', item.identifier)
  item.status = 'latest'
  await handleRefreshLocal()
}

function getSourceTypeIcon(sourceType: string) {
  if (sourceType === 'local') {
    return 'mdi-folder-open'
  } else if (sourceType === 'remote') {
    return 'mdi-cloud'
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
    ShowError(err)
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

  const srv = FileListService.getInstance()
  try {
    for (const identifier of localListSelected.value) {
      if (identifier in filelistStore.downloadedFile) {
        await srv.removeDownloaded(identifier)
      } else if (identifier in filelistStore.localFile) {
        await srv.removeLocal(identifier)
      }
    }
  } catch (err) {
    ShowError(err)
    return
  }
  ShowInfo('Selected files deleted.')
}

onMounted(async () => {
  try {
    // load local file list
    await filelistStore.refreshLocalSource()
    // // load remote file list if not loaded yet
    // if (Object.keys(filelistStore.remoteManifest).length === 0) {
    //   await filelistStore.fetchRemoteSource()
    // }
  } catch (err) {
    ShowError(err)
  }
})
</script>

<template>
  <div class="root">
    <div class="full-width">
      <v-btn class="full-width text-none" prepend-icon="mdi-wrench" @click="showMenu = true">
        Manage File List
      </v-btn>
    </div>

    <FileNameTableSelector v-model="selectedValue"></FileNameTableSelector>
  </div>

  <v-dialog v-model="showMenu" width="auto">
    <v-card class="manage-dialog">
      <v-card-text>
        <div class="header-bar">
          <h6 class="text-h6">Manage File List</h6>
          <v-btn icon="mdi-close" flat density="comfortable" @click="showMenu = false"></v-btn>
        </div>

        <div class="btn-row">
          <v-btn class="text-none" prepend-icon="mdi-folder-open" @click="handleOpenLocalDir"
            >Open Local Dir</v-btn
          >
          <v-btn
            class="text-none"
            prepend-icon="mdi-cloud-download"
            :loading="fetchingRemote"
            @click="handleFetchRemote"
            >Fetch Remote</v-btn
          >
        </div>

        <SplitPanel v-model:leftWidth="leftPanelWidth" v-model:rightWidth="rightPanelWidth">
          <template #left>
            <div class="table-container">
              <h6 class="text-h6 ml-2 mt-2 mr-2">Local</h6>
              <v-data-table
                class="local-list"
                v-model="localListSelected"
                :headers="headers"
                :items="localSources"
                item-value="identifier"
                show-select
                fixed-header
                height="400"
              >
                <template v-slot:item.identifier="{ item }">
                  <span> {{ item.identifier }}</span>
                  <v-icon class="ml-2" :icon="getSourceTypeIcon(item.sourceType)" small></v-icon>
                </template>

                <template v-slot:bottom>
                  <div class="button-group">
                    <v-btn class="text-none" prepend-icon="mdi-refresh" @click="handleRefreshLocal"
                      >Refresh</v-btn
                    >
                    <v-btn class="text-none" prepend-icon="mdi-delete" @click="handleDeleteLocal"
                      >Delete</v-btn
                    >
                  </div>
                </template>
              </v-data-table>
            </div>
          </template>
          <template #right>
            <div class="right-panel-content">
              <div class="cloud-list">
                <h6 class="text-h6 mb-4">Downloadable</h6>
                <v-list density="compact">
                  <v-list-item
                    v-for="item in downloadableItems"
                    :key="item.identifier"
                    :title="item.identifier"
                    :subtitle="`${item.updateTime.toLocaleString()}`"
                  >
                    <template v-slot:append>
                      <!-- download button -->
                      <v-btn
                        v-if="['downloadable', 'downloading'].includes(item.status)"
                        :disabled="item.status === 'downloading'"
                        size="small"
                        variant="tonal"
                        color="primary"
                        icon="mdi-download"
                        @click="handleDownload(item)"
                      ></v-btn>
                      <!-- update button -->
                      <v-btn
                        v-if="['updateable', 'updating'].includes(item.status)"
                        :disabled="item.status === 'updating'"
                        size="small"
                        variant="tonal"
                        color="warning"
                        icon="mdi-update"
                        @click="handleUpdateItem(item)"
                      ></v-btn>
                      <!-- notify if conflict with local file -->
                      <v-tooltip v-if="item.status === 'conflict'">
                        <template v-slot:activator="{ props }">
                          <v-icon v-bind="props" icon="mdi-alert-circle" color="warning"> </v-icon>
                        </template>
                        <div>
                          <span>You have a local file with same identifier.</span> <br />
                          <span
                            >If you want to download, please rename or delete the local file
                            first.</span
                          >
                        </div>
                      </v-tooltip>
                    </template>
                  </v-list-item>
                </v-list>
              </div>
            </div>
          </template>
        </SplitPanel>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<style scoped lang="scss">
.root {
  display: flex;
  flex-direction: column;
  row-gap: 16px;
}

.full-width {
  width: 100%;
}

.manage-dialog {
  min-width: 650px;
  width: 80vw;
}

.header-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.btn-row {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  margin-bottom: 0.8rem;
}

li {
  list-style: none;
}

.table-container {
  height: 100%;
}

.right-panel-content {
  height: 100%;
}

.table-container {
  flex: 1;
  overflow: hidden;
}

.local-list {
  overflow-y: auto;
}

.button-group {
  display: flex;
  gap: 8px;
  padding: 8px;
  background: white;
  position: sticky;
  bottom: 0;
  z-index: 1;
}

.right-panel {
  flex: 3;
  border-left: 1px solid #ddd;
  padding-left: 16px;
  overflow-y: auto;
  max-height: 400px;

  .cloud-list,
  .update-section {
    padding: 8px;
  }
}
</style>
