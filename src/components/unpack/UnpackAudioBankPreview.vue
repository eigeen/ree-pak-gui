<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  Download,
  LoaderCircle,
  Pause,
  Play,
  Repeat,
  Search,
  Square,
  Volume2,
  VolumeX
} from 'lucide-vue-next'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  audio_extract_wavs,
  audio_list_container,
  type AudioContainerInfo,
  type AudioEntryInfo
} from '@/api/tauri/pak'
import AppCursorContextMenu from '@/components/context-menu/AppCursorContextMenu.vue'
import {
  useAudioBankExportProgress,
  type AudioExportFormat
} from '@/composables/useAudioBankExportProgress'
import { getAudioSourceRef, resolveAudioBankDirectoryName } from '@/lib/audioBank'
import type { ContextMenuEntry } from '@/lib/contextMenu'
import type { ExplorerEntry } from '@/lib/unpackExplorer'
import { ShowError, ShowInfo } from '@/utils/message'

const props = defineProps<{
  entry: ExplorerEntry
}>()

const { t } = useI18n()

const PLAYBACK_RATES = [0.5, 0.75, 1, 1.25, 1.5, 2]

const audioRef = ref<HTMLAudioElement | null>(null)
const progressRef = ref<HTMLElement | null>(null)
const volumeRef = ref<HTMLElement | null>(null)

const containerInfo = ref<AudioContainerInfo | null>(null)
const loadingContainer = ref(false)
const wavUrls = ref<Record<number, string>>({})
const durations = ref<Record<number, number>>({})

const preparingIndex = ref<number | null>(null)
const currentIndex = ref<number | null>(null)
const isPlaying = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const volume = ref(1)
const isMuted = ref(false)
const playbackRate = ref(1)
const loop = ref(false)
const searchText = ref('')
const entryContextMenuTarget = ref<AudioEntryInfo | null>(null)
const entryContextMenuOpen = ref(false)
const entryContextMenuPosition = ref({ x: 0, y: 0 })

const source = computed(() => getAudioSourceRef(props.entry))

const filteredEntries = computed(() => {
  const list = containerInfo.value?.entries ?? []
  const query = searchText.value.trim().toLowerCase()
  if (!query) return list
  return list.filter((entry) => {
    const indexText = formatIndex(entry.index)
    return (
      indexText.includes(query) ||
      String(entry.wemId).toLowerCase().includes(query) ||
      (entry.languageId !== null &&
        entry.languageId !== undefined &&
        String(entry.languageId).includes(query))
    )
  })
})

const hasLanguage = computed(() =>
  (containerInfo.value?.entries ?? []).some(
    (entry) => entry.languageId !== null && entry.languageId !== undefined
  )
)

const currentEntry = computed<AudioEntryInfo | null>(() => {
  if (currentIndex.value === null) return null
  return containerInfo.value?.entries.find((entry) => entry.index === currentIndex.value) ?? null
})

const entryContextMenuItems = computed(() =>
  buildEntryContextMenuItems(entryContextMenuTarget.value)
)

function buildEntryContextMenuItems(entry: AudioEntryInfo | null): ContextMenuEntry[] {
  if (!entry) return []

  return [
    buildPlayEntryAction(entry),
    {
      type: 'separator',
      key: 'audio-preview-export-separator'
    },
    buildExportEntryAction(entry, 'wem'),
    buildExportEntryAction(entry, 'wav')
  ]
}

function buildPlayEntryAction(entry: AudioEntryInfo): ContextMenuEntry {
  return {
    type: 'action',
    key: 'audio-preview-play',
    label: t('unpack.audioBankPlay'),
    icon: Play,
    disabled: preparingIndex.value !== null,
    action: () => void playEntry(entry)
  }
}

function buildExportEntryAction(
  entry: AudioEntryInfo,
  format: AudioExportFormat
): ContextMenuEntry {
  return {
    type: 'action',
    key: `audio-preview-export-${format}`,
    label: t(`unpack.exportAudioAs${format === 'wem' ? 'Wem' : 'Wav'}`),
    icon: Download,
    disabled: exporting.value,
    action: () => void exportSingleEntry(entry, format)
  }
}

const progressRatio = computed(() => {
  if (!duration.value || !Number.isFinite(duration.value)) return 0
  return Math.min(1, Math.max(0, currentTime.value / duration.value))
})

const containerExtension = computed(() => {
  const path = containerInfo.value?.sourcePath ?? ''
  const match = path.match(/\.([a-z0-9]+)(?:\.\d+(?:\.[A-Za-z0-9]+)?)?$/i)
  if (match && match[1]) return match[1].toUpperCase()
  return (containerInfo.value?.containerKind ?? '').toUpperCase()
})

const { exporting, exportEntries } = useAudioBankExportProgress({
  source,
  getBankDirectoryName: getAudioBankDirectoryName
})

watch(
  () => props.entry.id,
  () => {
    void resetForNewSource()
  },
  { immediate: true }
)

watch(volume, (value) => {
  const audio = audioRef.value
  if (audio) audio.volume = value
  if (value > 0 && isMuted.value) isMuted.value = false
})

watch(isMuted, (value) => {
  const audio = audioRef.value
  if (audio) audio.muted = value
})

watch(playbackRate, (value) => {
  const audio = audioRef.value
  if (audio) audio.playbackRate = value
})

watch(loop, (value) => {
  const audio = audioRef.value
  if (audio) audio.loop = value
})

async function resetForNewSource() {
  stopPlayback()
  containerInfo.value = null
  wavUrls.value = {}
  durations.value = {}
  currentIndex.value = null
  duration.value = 0
  currentTime.value = 0
  searchText.value = ''
  await loadContainer()
}

async function loadContainer() {
  if (!source.value) {
    ShowError(t('unpack.audioBankMissingSource'))
    return
  }

  loadingContainer.value = true
  try {
    containerInfo.value = await audio_list_container(source.value)
  } catch (error) {
    ShowError(error instanceof Error ? error.message : String(error))
  } finally {
    loadingContainer.value = false
  }
}

async function ensurePrepared(entry: AudioEntryInfo): Promise<string | null> {
  const cached = wavUrls.value[entry.index]
  if (cached) return cached
  if (!source.value) return null

  preparingIndex.value = entry.index
  try {
    const [wavPath] = await audio_extract_wavs({
      source: source.value,
      indices: [entry.index]
    })
    if (!wavPath) return null
    const url = convertFileSrc(wavPath, 'asset')
    wavUrls.value = { ...wavUrls.value, [entry.index]: url }
    return url
  } catch (error) {
    ShowError(error instanceof Error ? error.message : String(error))
    return null
  } finally {
    preparingIndex.value = null
  }
}

async function playEntry(entry: AudioEntryInfo) {
  const url = await ensurePrepared(entry)
  if (!url) return

  const audio = audioRef.value
  if (!audio) return

  if (currentIndex.value !== entry.index) {
    audio.src = url
    audio.currentTime = 0
    currentIndex.value = entry.index
    duration.value = durations.value[entry.index] ?? 0
    currentTime.value = 0
  }

  await nextTick()
  try {
    await audio.play()
  } catch (error) {
    ShowError(error instanceof Error ? error.message : String(error))
  }
}

async function togglePlay() {
  const audio = audioRef.value
  if (!audio) return

  if (currentIndex.value === null) {
    const first = filteredEntries.value[0] ?? containerInfo.value?.entries[0]
    if (first) await playEntry(first)
    return
  }

  if (audio.paused) {
    try {
      await audio.play()
    } catch (error) {
      ShowError(error instanceof Error ? error.message : String(error))
    }
  } else {
    audio.pause()
  }
}

function stopPlayback() {
  const audio = audioRef.value
  if (!audio) return
  audio.pause()
  audio.currentTime = 0
  isPlaying.value = false
  currentTime.value = 0
}

function releaseAudioFileReferences() {
  const audio = audioRef.value
  if (!audio) return

  audio.pause()
  audio.removeAttribute('src')
  audio.load()
  wavUrls.value = {}
}

function cycleSpeed() {
  const idx = PLAYBACK_RATES.indexOf(playbackRate.value)
  const next = PLAYBACK_RATES[(idx + 1) % PLAYBACK_RATES.length] ?? 1
  playbackRate.value = next
}

function toggleMute() {
  isMuted.value = !isMuted.value
}

function handleProgressClick(event: MouseEvent) {
  const audio = audioRef.value
  const bar = progressRef.value
  if (!audio || !bar || !duration.value || !Number.isFinite(duration.value)) return
  const rect = bar.getBoundingClientRect()
  const ratio = Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width))
  audio.currentTime = ratio * duration.value
}

function handleVolumeClick(event: MouseEvent) {
  const bar = volumeRef.value
  if (!bar) return
  const rect = bar.getBoundingClientRect()
  const ratio = Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width))
  volume.value = ratio
}

function onTimeUpdate() {
  const audio = audioRef.value
  if (!audio) return
  currentTime.value = audio.currentTime
}

function onLoadedMetadata() {
  const audio = audioRef.value
  if (!audio || currentIndex.value === null) return
  if (Number.isFinite(audio.duration)) {
    duration.value = audio.duration
    durations.value = { ...durations.value, [currentIndex.value]: audio.duration }
  }
}

function onPlay() {
  isPlaying.value = true
}

function onPause() {
  isPlaying.value = false
}

function onEnded() {
  isPlaying.value = false
  if (!loop.value) currentTime.value = duration.value
}

async function exportCurrent() {
  if (currentIndex.value === null || !currentEntry.value) {
    ShowInfo(t('unpack.audioBankNoCurrent'))
    return
  }
  await exportEntries([currentEntry.value])
}

async function exportAll() {
  const entries = containerInfo.value?.entries
  if (!entries?.length) return
  await exportEntries(entries, { createBankDirectory: true })
}

async function exportSingleEntry(entry: AudioEntryInfo, format: AudioExportFormat) {
  await exportEntries([entry], { format })
}

function getAudioBankDirectoryName() {
  return resolveAudioBankDirectoryName(containerInfo.value?.sourcePath, props.entry.name)
}

function formatBytes(size: number) {
  if (!Number.isFinite(size) || size < 0) return '—'
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / 1024 / 1024).toFixed(1)} MB`
}

function formatTime(seconds: number) {
  if (!Number.isFinite(seconds) || seconds < 0) return '—'
  const total = Math.floor(seconds)
  const mm = Math.floor(total / 60)
  const ss = total % 60
  return `${String(mm).padStart(2, '0')}:${String(ss).padStart(2, '0')}`
}

function formatIndex(index: number) {
  return String(index).padStart(3, '0')
}

function handleEntryContextMenu(entry: AudioEntryInfo, event: MouseEvent) {
  event.preventDefault()
  entryContextMenuTarget.value = entry
  entryContextMenuPosition.value = {
    x: event.clientX,
    y: event.clientY
  }
  entryContextMenuOpen.value = true
}

onUnmounted(releaseAudioFileReferences)
</script>

<template>
  <div class="audio-preview flex h-full min-w-0 flex-col bg-[var(--surface-panel)] text-foreground">
    <!-- transport bar -->
    <div
      class="flex h-15 shrink-0 items-center gap-3 border-b border-border/80 bg-[var(--surface-toolbar)] px-4 py-2.5"
    >
      <button
        type="button"
        class="ap-icon-btn"
        :disabled="currentIndex === null"
        @click="stopPlayback"
      >
        <Square class="size-3" />
      </button>
      <button
        type="button"
        class="inline-flex size-9 shrink-0 items-center justify-center rounded-full bg-[var(--audio-accent)] text-[#1a1a1a] transition-colors hover:bg-[var(--audio-accent-hover)] disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="loadingContainer || !containerInfo?.entries.length"
        @click="togglePlay"
      >
        <LoaderCircle v-if="preparingIndex !== null" class="size-4 animate-spin" />
        <Pause v-else-if="isPlaying" class="size-4" />
        <Play v-else class="size-4" />
      </button>

      <div class="flex min-w-0 flex-1 items-center gap-2">
        <span class="shrink-0 text-[11px] text-muted-foreground tabular-nums">
          {{ formatTime(currentTime) }}
        </span>
        <div
          ref="progressRef"
          class="h-1.5 min-w-0 flex-1 cursor-pointer overflow-hidden rounded-[3px] bg-secondary"
          @click="handleProgressClick"
        >
          <div
            class="h-full bg-[var(--audio-accent)] transition-[width] duration-75 ease-linear"
            :style="{ width: progressRatio * 100 + '%' }"
          />
        </div>
        <span class="shrink-0 text-[11px] text-muted-foreground tabular-nums">
          {{ formatTime(duration) }}
        </span>
      </div>

      <button
        type="button"
        class="ap-icon-btn"
        :class="{ 'text-[var(--audio-accent)]': loop }"
        @click="loop = !loop"
      >
        <Repeat class="size-3.5" />
      </button>

      <button
        type="button"
        class="inline-flex h-6 shrink-0 items-center justify-center rounded bg-secondary px-2 text-[11px] tabular-nums text-foreground transition-colors hover:bg-accent"
        @click="cycleSpeed"
      >
        {{ playbackRate.toFixed(playbackRate % 1 === 0 ? 1 : 2) }}x
      </button>

      <div class="flex shrink-0 items-center gap-1.5">
        <button
          type="button"
          class="inline-flex items-center justify-center p-0.5 text-muted-foreground hover:text-foreground"
          @click="toggleMute"
        >
          <VolumeX v-if="isMuted || volume === 0" class="size-3.5" />
          <Volume2 v-else class="size-3.5" />
        </button>
        <div
          ref="volumeRef"
          class="h-1 w-16 cursor-pointer overflow-hidden rounded-[2px] bg-secondary"
          @click="handleVolumeClick"
        >
          <div
            class="h-full bg-foreground/70"
            :style="{ width: (isMuted ? 0 : volume) * 100 + '%' }"
          />
        </div>
      </div>

      <button
        type="button"
        class="ap-export-btn"
        :disabled="exporting || currentIndex === null"
        @click="exportCurrent"
      >
        <LoaderCircle v-if="exporting" class="size-3 animate-spin" />
        <Download v-else class="size-3" />
        <span>{{ t('unpack.audioBankExportCurrent') }}</span>
      </button>
    </div>

    <!-- now-playing strip -->
    <div
      v-if="currentEntry"
      class="flex h-8 shrink-0 items-center gap-2.5 overflow-hidden border-b border-border/80 px-4 py-1.5 text-[11px]"
    >
      <Play class="size-3 text-[var(--audio-accent)]" />
      <span class="text-muted-foreground">{{ t('unpack.audioBankNowPlaying') }}</span>
      <span class="tabular-nums text-muted-foreground">#{{ formatIndex(currentEntry.index) }}</span>
      <span class="text-xs font-semibold text-[var(--audio-accent-soft)]">
        WEM {{ currentEntry.wemId }}
      </span>
      <span class="text-muted-foreground/60">·</span>
      <span class="text-muted-foreground">{{ formatBytes(currentEntry.size) }}</span>
      <template v-if="containerExtension">
        <span class="text-muted-foreground/60">·</span>
        <span class="text-muted-foreground">{{ containerExtension }}</span>
      </template>
      <template v-if="currentEntry.languageId !== null && currentEntry.languageId !== undefined">
        <span class="text-muted-foreground/60">·</span>
        <span class="text-muted-foreground">Lang {{ currentEntry.languageId }}</span>
      </template>
    </div>

    <!-- table toolbar -->
    <div
      class="flex h-9 shrink-0 items-center gap-2.5 border-b border-border/80 bg-[var(--surface-toolbar)] px-3 py-1.5"
    >
      <div
        class="flex w-60 items-center gap-1.5 rounded-[3px] border border-border/80 bg-[var(--surface-canvas)] px-2 py-1"
      >
        <Search class="size-3 text-muted-foreground" />
        <input
          v-model="searchText"
          type="text"
          class="min-w-0 flex-1 border-none bg-transparent text-[11px] text-foreground outline-none placeholder:text-muted-foreground"
          :placeholder="t('unpack.audioBankSearchPlaceholder')"
        />
      </div>
      <span class="shrink-0 rounded-[3px] bg-secondary px-2 py-1 text-[11px] text-muted-foreground">
        {{ t('unpack.audioBankTrackCount', { count: filteredEntries.length }) }}
      </span>
      <div class="flex-1" />
      <button
        type="button"
        class="ap-export-btn"
        :disabled="exporting || !containerInfo?.entries.length"
        @click="exportAll"
      >
        <LoaderCircle v-if="exporting" class="size-3 animate-spin" />
        <Download v-else class="size-3" />
        <span>{{ t('unpack.audioBankExportAll') }}</span>
      </button>
    </div>

    <!-- table header -->
    <div
      class="ap-tbl-grid h-7.5 shrink-0 gap-2.5 border-b border-border/80 bg-[var(--surface-canvas)] px-2.5 text-[11px] font-semibold text-foreground"
    >
      <div class="ap-col-marker" />
      <div class="ap-col-index">{{ t('unpack.audioBankColumnIndex') }}</div>
      <div class="ap-col-id">{{ t('unpack.audioBankColumnId') }}</div>
      <div class="ap-col-duration">{{ t('unpack.audioBankColumnDuration') }}</div>
      <div class="ap-col-size">{{ t('unpack.audioBankColumnSize') }}</div>
      <div v-if="hasLanguage" class="ap-col-lang">
        {{ t('unpack.audioBankColumnLang') }}
      </div>
    </div>

    <!-- table body -->
    <div class="editor-scrollbar min-h-0 flex-1 overflow-auto bg-[var(--surface-panel)]">
      <div
        v-if="loadingContainer"
        class="flex h-full items-center justify-center p-8 text-xs text-muted-foreground"
      >
        <LoaderCircle class="mr-2 size-4 animate-spin text-[var(--audio-accent)]" />
        {{ t('unpack.audioBankLoading') }}
      </div>
      <div
        v-else-if="!containerInfo?.entries.length"
        class="flex h-full items-center justify-center p-8 text-xs text-muted-foreground"
      >
        {{ t('unpack.audioBankEmpty') }}
      </div>
      <div
        v-else
        v-for="entry in filteredEntries"
        :key="entry.index"
        class="ap-tbl-grid ap-tbl-row h-7 shrink-0 cursor-pointer gap-2.5 border-b border-border/40 border-l-2 border-l-transparent px-2.5 text-[11px] text-muted-foreground transition-colors duration-100 hover:bg-secondary/40 hover:text-foreground"
        :data-active="entry.index === currentIndex || null"
        @click="playEntry(entry)"
        @contextmenu="handleEntryContextMenu(entry, $event)"
      >
        <div class="ap-col-marker">
          <LoaderCircle
            v-if="preparingIndex === entry.index"
            class="size-3 animate-spin text-[var(--audio-accent)]"
          />
          <Play
            v-else-if="entry.index === currentIndex"
            class="size-3 text-[var(--audio-accent)]"
          />
        </div>
        <div class="ap-col-index">{{ formatIndex(entry.index) }}</div>
        <div class="ap-col-id">{{ entry.wemId }}</div>
        <div class="ap-col-duration">
          {{ durations[entry.index] !== undefined ? formatTime(durations[entry.index]!) : '—' }}
        </div>
        <div class="ap-col-size">{{ formatBytes(entry.size) }}</div>
        <div v-if="hasLanguage" class="ap-col-lang">
          {{ entry.languageId !== null && entry.languageId !== undefined ? entry.languageId : '—' }}
        </div>
      </div>
    </div>

    <audio
      ref="audioRef"
      preload="metadata"
      @timeupdate="onTimeUpdate"
      @loadedmetadata="onLoadedMetadata"
      @play="onPlay"
      @pause="onPause"
      @ended="onEnded"
    />

    <AppCursorContextMenu
      :items="entryContextMenuItems"
      :open="entryContextMenuOpen"
      :x="entryContextMenuPosition.x"
      :y="entryContextMenuPosition.y"
      @update:open="entryContextMenuOpen = $event"
    />
  </div>
</template>

<style scoped>
.audio-preview {
  --audio-accent: #ffad66;
  --audio-accent-hover: #ffbe85;
  --audio-accent-soft: #ffd3a9;
}

.ap-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  flex-shrink: 0;
  border-radius: 9999px;
  background: var(--secondary);
  color: var(--foreground);
  transition:
    background 0.15s ease,
    color 0.15s ease,
    opacity 0.15s ease;
}

.ap-export-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  flex-shrink: 0;
  border-radius: 0.25rem;
  background: var(--secondary);
  color: var(--foreground);
  padding: 0.375rem 0.75rem;
  font-size: 11px;
  transition:
    background 0.15s ease,
    opacity 0.15s ease;
}

.ap-icon-btn:hover:not(:disabled),
.ap-export-btn:hover:not(:disabled) {
  background: var(--accent);
}

.ap-icon-btn:disabled,
.ap-export-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.ap-icon-btn:disabled {
  opacity: 0.4;
}

.ap-tbl-grid {
  display: grid;
  grid-template-columns: 24px 48px minmax(0, 1fr) 80px 80px;
  align-items: center;
}

.ap-tbl-grid:has(.ap-col-lang) {
  grid-template-columns: 24px 48px minmax(0, 1fr) 80px 80px 64px;
}

.ap-col-marker {
  display: flex;
  align-items: center;
  justify-content: center;
}

.ap-col-index {
  color: var(--muted-foreground);
  font-variant-numeric: tabular-nums;
  text-align: center;
}

.ap-col-id {
  min-width: 0;
  overflow: hidden;
  color: var(--foreground);
  font-size: 0.75rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ap-col-duration,
.ap-col-size,
.ap-col-lang {
  font-variant-numeric: tabular-nums;
}

.ap-tbl-row[data-active] {
  background: color-mix(in srgb, var(--audio-accent) 14%, transparent);
  border-left-color: var(--audio-accent);
}

.ap-tbl-row[data-active] .ap-col-id,
.ap-tbl-row[data-active] .ap-col-duration,
.ap-tbl-row[data-active] .ap-col-size,
.ap-tbl-row[data-active] .ap-col-lang {
  color: var(--audio-accent-soft);
  font-weight: 600;
}

.ap-tbl-row[data-active] .ap-col-index {
  color: var(--audio-accent);
  font-weight: 600;
}
</style>
