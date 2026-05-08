<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { AudioLines, LoaderCircle, Music, Play } from 'lucide-vue-next'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  audio_extract_wavs,
  audio_list_container,
  type AudioContainerInfo,
  type AudioEntryInfo
} from '@/api/tauri/pak'
import { Button } from '@/components/ui/button'
import type { ExplorerEntry } from '@/lib/unpackExplorer'
import { ShowError } from '@/utils/message'

const props = defineProps<{
  entry: ExplorerEntry
}>()

const { t } = useI18n()

const loading = ref(false)
const loadingIndex = ref<number | null>(null)
const containerInfo = ref<AudioContainerInfo | null>(null)
const wavUrls = ref<Record<number, string>>({})

const canLoadAudio = computed(() => Boolean(props.entry.hash && props.entry.belongsTo))
const source = computed(() => {
  if (!props.entry.hash || !props.entry.belongsTo) return null
  return {
    hash: props.entry.hash,
    belongsTo: props.entry.belongsTo
  }
})

watch(
  () => props.entry.id,
  () => {
    void loadContainer(true)
  },
  { immediate: true }
)

async function loadContainer(reset: boolean = false) {
  if (reset) {
    containerInfo.value = null
    wavUrls.value = {}
  }

  if (!source.value) {
    ShowError(t('unpack.audioBankMissingSource'))
    return
  }

  loading.value = true
  try {
    containerInfo.value = await audio_list_container(source.value)
    wavUrls.value = {}
  } catch (error) {
    ShowError(error instanceof Error ? error.message : String(error))
  } finally {
    loading.value = false
  }
}

async function prepareWav(entry: AudioEntryInfo) {
  if (!source.value || loadingIndex.value !== null) return
  if (wavUrls.value[entry.index]) return

  loadingIndex.value = entry.index
  try {
    const [wavPath] = await audio_extract_wavs({
      source: source.value,
      indices: [entry.index]
    })
    if (wavPath) {
      wavUrls.value = {
        ...wavUrls.value,
        [entry.index]: convertFileSrc(wavPath, 'asset')
      }
    }
  } catch (error) {
    ShowError(error instanceof Error ? error.message : String(error))
  } finally {
    loadingIndex.value = null
  }
}

function formatBytes(size: number) {
  if (!Number.isFinite(size)) return '-'
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / 1024 / 1024).toFixed(1)} MB`
}
</script>

<template>
  <div class="flex h-full min-w-0 flex-col bg-[#0a0a0a] text-foreground">
    <div class="flex items-center justify-between gap-4 border-b border-[#16343a] px-5 py-4">
      <div class="min-w-0">
        <p class="flex items-center gap-2 text-sm font-semibold text-foreground">
          <Music class="size-4 text-[#5dccd0]" />
          {{ t('unpack.audioBankPreviewTitle') }}
        </p>
        <p class="mt-1 truncate text-xs text-muted-foreground">
          {{ containerInfo?.sourcePath ?? entry.name }}
        </p>
      </div>
      <Button
        size="sm"
        variant="outline"
        class="shrink-0"
        :disabled="loading || !canLoadAudio"
        @click="loadContainer"
      >
        <LoaderCircle v-if="loading" class="size-4 animate-spin" />
        <AudioLines v-else class="size-4" />
        {{ t('unpack.audioBankReload') }}
      </Button>
    </div>

    <div class="editor-scrollbar min-h-0 flex-1 overflow-auto p-5">
      <div v-if="loading" class="flex h-full items-center justify-center text-sm text-muted-foreground">
        <LoaderCircle class="mr-2 size-4 animate-spin text-[#5dccd0]" />
        {{ t('unpack.audioBankLoading') }}
      </div>

      <div v-else-if="!containerInfo?.entries.length" class="flex h-full items-center justify-center p-8">
        <div class="flex max-w-md flex-col items-center gap-3 text-center">
          <div class="flex size-14 items-center justify-center rounded-full bg-[#15282d]">
            <Music class="size-6 text-[#5dccd0]" />
          </div>
          <p class="text-sm text-muted-foreground">
            {{
              containerInfo
                ? t('unpack.audioBankEmpty')
                : t('unpack.audioBankPreviewPlaceholder')
            }}
          </p>
        </div>
      </div>

      <div v-else class="grid gap-2">
        <div
          v-for="audioEntry in containerInfo.entries"
          :key="audioEntry.index"
          class="grid min-h-20 grid-cols-[minmax(0,1fr)_auto] items-center gap-4 border border-[#16343a] bg-[#0f1517] px-4 py-3"
        >
          <div class="min-w-0">
            <div class="flex min-w-0 items-center gap-2">
              <AudioLines class="size-4 shrink-0 text-[#5dccd0]" />
              <p class="truncate text-sm font-semibold text-foreground">
                {{ t('unpack.audioBankEntryTitle', { index: audioEntry.index + 1 }) }}
              </p>
            </div>
            <p class="mt-1 text-xs text-muted-foreground">
              WEM {{ audioEntry.wemId }} · {{ formatBytes(audioEntry.size) }}
              <template v-if="audioEntry.languageId !== null && audioEntry.languageId !== undefined">
                · Lang {{ audioEntry.languageId }}
              </template>
            </p>
            <audio
              v-if="wavUrls[audioEntry.index]"
              class="mt-3 h-9 w-full"
              controls
              :src="wavUrls[audioEntry.index]"
            />
          </div>

          <Button
            size="sm"
            variant="secondary"
            class="w-28"
            :disabled="loadingIndex !== null"
            @click="prepareWav(audioEntry)"
          >
            <LoaderCircle
              v-if="loadingIndex === audioEntry.index"
              class="size-4 animate-spin"
            />
            <Play v-else class="size-4" />
            {{
              wavUrls[audioEntry.index]
                ? t('unpack.audioBankReady')
                : t('unpack.audioBankPrepare')
            }}
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
