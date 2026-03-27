<template>
  <div class="mx-auto flex max-w-4xl flex-col gap-6">
    <div class="space-y-2">
      <p class="section-eyebrow">Tool</p>
      <h3 class="section-title">{{ t('pathScanner.title') }}</h3>
      <p class="section-copy">{{ t('pathScanner.description') }}</p>
    </div>

    <div class="app-panel-muted flex items-start gap-3 p-4">
      <div
        class="mt-0.5 flex size-9 shrink-0 items-center justify-center rounded-2xl border border-primary/20 bg-primary/10 text-primary"
      >
        <Info class="size-4" />
      </div>
      <p class="text-sm leading-6 text-muted-foreground">
        {{ t('pathScanner.lightVersionTip') }}
        <button
          class="mx-1 font-medium text-primary transition hover:text-primary/80"
          type="button"
          @click="openUrl('https://github.com/eigeen/ree-path-searcher')"
        >
          {{ t('pathScanner.standaloneVersion') }}
        </button>
        {{ t('pathScanner.betterPerformance') }}
      </p>
    </div>

    <div class="grid gap-6">
      <section class="app-panel p-5">
        <div class="mb-4 flex items-center justify-between gap-3">
          <div>
            <p class="section-eyebrow">{{ t('pathScanner.pakFiles') }}</p>
            <h4 class="section-title">{{ t('pathScanner.pakFiles') }}</h4>
          </div>
          <Button :disabled="scanning" @click="selectPakFiles">
            <FolderPlus class="size-4" />
            {{ t('pathScanner.selectPakFiles') }}
          </Button>
        </div>

        <div class="app-panel-muted p-3">
          <div v-if="pakFiles.length === 0" class="empty-state min-h-32">
            <p class="text-sm font-medium text-foreground">{{ t('pathScanner.selectPakFiles') }}</p>
            <p class="section-copy">支持选择多个 `.pak` 文件参与扫描。</p>
          </div>

          <div v-else class="space-y-2">
            <div
              v-for="(file, index) in pakFiles"
              :key="`${file}-${index}`"
              class="flex items-center gap-3 rounded-2xl border border-border/70 bg-background/85 px-3 py-3"
            >
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-medium text-foreground">{{ file }}</p>
              </div>
              <Button
                size="icon-sm"
                variant="ghost"
                class="rounded-full"
                :disabled="scanning"
                @click="removePakFile(index)"
              >
                <X class="size-4" />
              </Button>
            </div>
          </div>
        </div>
      </section>

      <section class="app-panel p-5">
        <div class="mb-4">
          <p class="section-eyebrow">{{ t('pathScanner.knownPathList') }}</p>
          <h4 class="section-title">{{ t('pathScanner.knownPathList') }}</h4>
        </div>
        <FileNameTableSelector v-model="selectedFileList" :items="comboItems" />
      </section>

      <section v-if="scanning || scanResult" class="app-panel p-5">
        <div class="mb-4">
          <p class="section-eyebrow">{{ t('pathScanner.scanStatus') }}</p>
          <h4 class="section-title">{{ t('pathScanner.scanStatus') }}</h4>
        </div>

        <div class="space-y-4">
          <Progress v-if="scanning" :model-value="45" class="h-2.5 rounded-full" />
          <p v-if="progressMessage" class="text-sm text-muted-foreground">{{ progressMessage }}</p>

          <div v-if="scanResult && !scanning" class="space-y-3">
            <p class="text-sm font-medium text-foreground">
              {{ t('pathScanner.scanComplete') }} {{ scanResult.length }}
              {{ t('pathScanner.foundPaths') }}
            </p>
            <Textarea
              :model-value="scanResult.join('\n')"
              class="min-h-64 font-mono text-xs"
              readonly
            />
            <Button variant="outline" @click="copyResults">
              <Copy class="size-4" />
              {{ t('pathScanner.copyResults') }}
            </Button>
          </div>
        </div>
      </section>
    </div>

    <div class="flex justify-end gap-3">
      <Button v-if="scanning" variant="destructive" @click="stopScan">
        <Square class="size-4" />
        {{ t('pathScanner.stopScan') }}
      </Button>
      <Button v-else :disabled="!canStartScan" @click="startScan">
        <Search class="size-4" />
        {{ t('pathScanner.startScan') }}
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Copy, FolderPlus, Info, Search, Square, X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { PathScanner } from '@/lib/pathScanner'
import { terminatePathScan, type PathScanOptions } from '@/api/tauri/tools'
import FileNameTableSelector from '@/components/FileNameTable/FileNameTableSelector.vue'
import { useFileListStore } from '@/store/filelist'
import { ShowError, ShowInfo } from '@/utils/message'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { Textarea } from '@/components/ui/textarea'

const { t } = useI18n()

const pakFiles = ref<string[]>([])
const selectedFileList = ref<string>('')
const fileListStore = useFileListStore()
const scanning = ref(false)
const progressMessage = ref('')
const scanResult = ref<string[] | null>(null)

let pathScanner: PathScanner | null = null

const canStartScan = computed(() => pakFiles.value.length > 0)

const resetState = () => {
  progressMessage.value = ''
  scanResult.value = null
}

const selectPakFiles = async () => {
  try {
    const selected = await openDialog({
      multiple: true,
      filters: [
        {
          name: 'Pak Files',
          extensions: ['pak']
        }
      ]
    })

    if (selected && Array.isArray(selected)) {
      pakFiles.value = [...pakFiles.value, ...selected]
    } else if (selected && typeof selected === 'string') {
      pakFiles.value.push(selected)
    }
  } catch (error) {
    ShowError(t('pathScanner.selectFilesFailed', { error }))
  }
}

const removePakFile = (index: number) => {
  pakFiles.value.splice(index, 1)
}

const startScan = async () => {
  if (!canStartScan.value) return

  resetState()
  scanning.value = true

  const options: PathScanOptions = {
    pakFiles: pakFiles.value,
    dumpFiles: []
  }

  try {
    pathScanner = new PathScanner((event) => {
      switch (event.event) {
        case 'startFile':
          progressMessage.value = `Scanning file ${event.data.current} / ${event.data.total}`
          break
        case 'finish':
          if (event.data.success) {
            scanResult.value = event.data.foundPaths
            progressMessage.value = 'Scan finished'
          } else {
            progressMessage.value = event.data.error ?? 'Unknown Error'
          }
          scanning.value = false
          break
      }
    })

    await pathScanner.scan(options)
  } catch (error) {
    scanning.value = false
    ShowError(t('pathScanner.scanFailed', { error }))
  }
}

const stopScan = async () => {
  if (pathScanner) {
    await terminatePathScan()
  }
  scanning.value = false
  progressMessage.value = 'Scan stopped'
}

const copyResults = async () => {
  if (!scanResult.value?.length) {
    return
  }

  try {
    await navigator.clipboard.writeText(scanResult.value.join('\n'))
    ShowInfo(t('pathScanner.resultsCopied'))
  } catch (error) {
    ShowError(t('pathScanner.copyFailed', { error }))
  }
}

const localSources = computed(() => {
  const itemsMap: Record<string, any> = {}

  for (const identifier in fileListStore.localFile) {
    const localFile = fileListStore.localFile[identifier]
    if (localFile) {
      itemsMap[identifier] = { ...localFile.source }
    }
  }

  for (const fileName in fileListStore.downloadedFile) {
    const downloaded = fileListStore.downloadedFile[fileName]
    const source = downloaded?.source
    if (source && !(source.identifier in itemsMap)) {
      itemsMap[source.identifier] = { ...source }
    }
  }

  return Object.values(itemsMap).sort((a: any, b: any) => a.identifier.localeCompare(b.identifier))
})

const comboItems = computed(() =>
  localSources.value.map((item: any) => ({
    label: item.identifier,
    value: item.identifier
  }))
)

onMounted(async () => {
  try {
    await fileListStore.refreshLocalSource()
  } catch (error) {
    console.error('Failed to load file list:', error)
  }
})
</script>
