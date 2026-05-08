<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { join } from '@tauri-apps/api/path'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
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
  type AudioEntryInfo,
  type AudioSourceRef
} from '@/api/tauri/pak'
import type { ExplorerEntry } from '@/lib/unpackExplorer'
import { getFileName } from '@/utils/path'
import { ShowError, ShowInfo } from '@/utils/message'

const props = defineProps<{
  entry: ExplorerEntry
}>()

const { t } = useI18n()

const PLAYBACK_RATES = [0.5, 0.75, 1, 1.25, 1.5, 2]
const FALLBACK_BANK_DIRECTORY = 'sound-bank'

type AudioExportOptions = {
  createBankDirectory?: boolean
}

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
const exporting = ref(false)

const source = computed(() => {
  if (!props.entry.hash || !props.entry.belongsTo) return null
  return {
    hash: props.entry.hash,
    belongsTo: props.entry.belongsTo
  }
})

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

async function exportEntries(entries: AudioEntryInfo[], options: AudioExportOptions = {}) {
  const exportSource = source.value
  if (!exportSource || exporting.value) return
  const outputDir = await chooseAudioExportOutputDir(options)
  if (!outputDir) return

  await runAudioExport(entries, outputDir, exportSource)
}

async function chooseAudioExportOutputDir(options: AudioExportOptions) {
  const target = await dialogOpen({
    directory: true,
    multiple: false,
    title: t('unpack.audioBankExportSelectDir')
  })
  if (typeof target !== 'string' || !target) return
  return await resolveExportOutputDir(target, options)
}

async function runAudioExport(
  entries: AudioEntryInfo[],
  outputDir: string,
  exportSource: AudioSourceRef
) {
  exporting.value = true
  try {
    await extractAndReportAudioEntries(entries, outputDir, exportSource)
  } catch (error) {
    ShowError(formatAudioExportError(error))
  } finally {
    exporting.value = false
  }
}

async function extractAndReportAudioEntries(
  entries: AudioEntryInfo[],
  outputDir: string,
  exportSource: AudioSourceRef
) {
  const paths = await audio_extract_wavs({
    source: exportSource,
    indices: entries.map((entry) => entry.index),
    outputDir
  })
  ShowInfo(t('unpack.audioBankExportDone', { count: paths.length }))
}

function formatAudioExportError(error: unknown) {
  const message = error instanceof Error ? error.message : String(error)
  return `${t('unpack.audioBankExportFailed')}: ${message}`
}

async function resolveExportOutputDir(target: string, options: AudioExportOptions) {
  if (!options.createBankDirectory) return target
  return await join(target, getAudioBankDirectoryName())
}

function getAudioBankDirectoryName() {
  const sourcePath = containerInfo.value?.sourcePath || props.entry.name
  const fileName = getFileName(sourcePath).trim()
  return sanitizeDirectoryName(fileName || FALLBACK_BANK_DIRECTORY)
}

function sanitizeDirectoryName(value: string) {
  const sanitized = value
    .replace(/[<>:"/\\|?*]/g, '_')
    .split('')
    .map(replaceControlCharacter)
    .join('')
    .replace(/[. ]+$/g, '')
    .trim()
  return sanitized || FALLBACK_BANK_DIRECTORY
}

function replaceControlCharacter(value: string) {
  return value.charCodeAt(0) < 32 ? '_' : value
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

onUnmounted(releaseAudioFileReferences)
</script>

<template>
  <div class="audio-preview">
    <div class="ap-transport">
      <button
        type="button"
        class="ap-icon-btn ap-stop"
        :disabled="currentIndex === null"
        @click="stopPlayback"
      >
        <Square class="size-3" />
      </button>
      <button
        type="button"
        class="ap-play"
        :disabled="loadingContainer || !containerInfo?.entries.length"
        @click="togglePlay"
      >
        <LoaderCircle
          v-if="preparingIndex !== null"
          class="size-4 animate-spin"
        />
        <Pause v-else-if="isPlaying" class="size-4" />
        <Play v-else class="size-4" />
      </button>

      <div class="ap-progress-wrap">
        <span class="ap-time">{{ formatTime(currentTime) }}</span>
        <div
          ref="progressRef"
          class="ap-progress"
          @click="handleProgressClick"
        >
          <div class="ap-progress-fill" :style="{ width: progressRatio * 100 + '%' }" />
        </div>
        <span class="ap-time">{{ formatTime(duration) }}</span>
      </div>

      <button
        type="button"
        class="ap-icon-btn"
        :class="{ 'ap-toggle-on': loop }"
        @click="loop = !loop"
      >
        <Repeat class="size-3.5" />
      </button>

      <button type="button" class="ap-speed" @click="cycleSpeed">
        {{ playbackRate.toFixed(playbackRate % 1 === 0 ? 1 : 2) }}x
      </button>

      <div class="ap-volume">
        <button type="button" class="ap-volume-btn" @click="toggleMute">
          <VolumeX v-if="isMuted || volume === 0" class="size-3.5" />
          <Volume2 v-else class="size-3.5" />
        </button>
        <div ref="volumeRef" class="ap-volume-bar" @click="handleVolumeClick">
          <div
            class="ap-volume-fill"
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

    <div v-if="currentEntry" class="ap-strip">
      <Play class="size-3 text-[#ffad66]" />
      <span class="ap-strip-label">{{ t('unpack.audioBankNowPlaying') }}</span>
      <span class="ap-strip-index">#{{ formatIndex(currentEntry.index) }}</span>
      <span class="ap-strip-name">WEM {{ currentEntry.wemId }}</span>
      <span class="ap-strip-sep">·</span>
      <span class="ap-strip-meta">{{ formatBytes(currentEntry.size) }}</span>
      <template v-if="containerExtension">
        <span class="ap-strip-sep">·</span>
        <span class="ap-strip-meta">{{ containerExtension }}</span>
      </template>
      <template
        v-if="currentEntry.languageId !== null && currentEntry.languageId !== undefined"
      >
        <span class="ap-strip-sep">·</span>
        <span class="ap-strip-meta">Lang {{ currentEntry.languageId }}</span>
      </template>
    </div>

    <div class="ap-tbl-toolbar">
      <div class="ap-search">
        <Search class="size-3 text-[#666666]" />
        <input
          v-model="searchText"
          type="text"
          :placeholder="t('unpack.audioBankSearchPlaceholder')"
        />
      </div>
      <span class="ap-count-chip">
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

    <div class="ap-tbl-hdr">
      <div class="ap-col-marker" />
      <div class="ap-col-index">{{ t('unpack.audioBankColumnIndex') }}</div>
      <div class="ap-col-id">{{ t('unpack.audioBankColumnId') }}</div>
      <div class="ap-col-duration">{{ t('unpack.audioBankColumnDuration') }}</div>
      <div class="ap-col-size">{{ t('unpack.audioBankColumnSize') }}</div>
      <div v-if="hasLanguage" class="ap-col-lang">
        {{ t('unpack.audioBankColumnLang') }}
      </div>
    </div>

    <div class="ap-tbl-body editor-scrollbar">
      <div
        v-if="loadingContainer"
        class="ap-empty"
      >
        <LoaderCircle class="mr-2 size-4 animate-spin text-[#ffad66]" />
        {{ t('unpack.audioBankLoading') }}
      </div>
      <div
        v-else-if="!containerInfo?.entries.length"
        class="ap-empty"
      >
        {{ t('unpack.audioBankEmpty') }}
      </div>
      <div
        v-else
        v-for="entry in filteredEntries"
        :key="entry.index"
        class="ap-tbl-row"
        :data-active="entry.index === currentIndex || null"
        @click="playEntry(entry)"
      >
        <div class="ap-col-marker">
          <LoaderCircle
            v-if="preparingIndex === entry.index"
            class="size-3 animate-spin text-[#ffad66]"
          />
          <Play
            v-else-if="entry.index === currentIndex"
            class="size-3 text-[#ffad66]"
          />
        </div>
        <div class="ap-col-index">{{ formatIndex(entry.index) }}</div>
        <div class="ap-col-id">{{ entry.wemId }}</div>
        <div class="ap-col-duration">
          {{ durations[entry.index] !== undefined ? formatTime(durations[entry.index]!) : '—' }}
        </div>
        <div class="ap-col-size">{{ formatBytes(entry.size) }}</div>
        <div v-if="hasLanguage" class="ap-col-lang">
          {{
            entry.languageId !== null && entry.languageId !== undefined
              ? entry.languageId
              : '—'
          }}
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
  </div>
</template>

<style scoped>
.audio-preview {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-width: 0;
  background: #1a1a1a;
  color: #cccccc;
  font-family: Inter, system-ui, sans-serif;
}

.ap-transport {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 60px;
  padding: 10px 16px;
  background: #1f1f1f;
  border-bottom: 1px solid #2a2a2a;
  flex-shrink: 0;
}

.ap-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 14px;
  background: #2a2a2a;
  color: #cccccc;
  cursor: pointer;
  transition:
    background 0.15s ease,
    color 0.15s ease;
  flex-shrink: 0;
}

.ap-icon-btn:hover:not(:disabled) {
  background: #3a3a3a;
}

.ap-icon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.ap-icon-btn.ap-toggle-on {
  color: #ffad66;
}

.ap-stop {
  color: #cccccc;
}

.ap-play {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 18px;
  background: #ffad66;
  color: #1a1a1a;
  cursor: pointer;
  transition: background 0.15s ease;
  flex-shrink: 0;
}

.ap-play:hover:not(:disabled) {
  background: #ffbe85;
}

.ap-play:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.ap-progress-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.ap-time {
  font-size: 11px;
  color: #aaaaaa;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.ap-progress {
  flex: 1;
  height: 6px;
  border-radius: 3px;
  background: #2a2a2a;
  overflow: hidden;
  cursor: pointer;
  min-width: 0;
}

.ap-progress-fill {
  height: 100%;
  background: #ffad66;
  transition: width 0.08s linear;
}

.ap-speed {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 4px 8px;
  height: 24px;
  border-radius: 4px;
  background: #2a2a2a;
  color: #cccccc;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  cursor: pointer;
  transition: background 0.15s ease;
  flex-shrink: 0;
}

.ap-speed:hover {
  background: #3a3a3a;
}

.ap-volume {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.ap-volume-btn {
  color: #aaaaaa;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 2px;
}

.ap-volume-btn:hover {
  color: #cccccc;
}

.ap-volume-bar {
  width: 64px;
  height: 4px;
  border-radius: 2px;
  background: #2a2a2a;
  overflow: hidden;
  cursor: pointer;
}

.ap-volume-fill {
  height: 100%;
  background: #cccccc;
}

.ap-export-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 4px;
  background: #2a2a2a;
  color: #cccccc;
  font-size: 11px;
  cursor: pointer;
  transition: background 0.15s ease;
  flex-shrink: 0;
}

.ap-export-btn:hover:not(:disabled) {
  background: #3a3a3a;
}

.ap-export-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.ap-strip {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 32px;
  padding: 6px 16px;
  background: #1a1a1a;
  border-bottom: 1px solid #2a2a2a;
  font-size: 11px;
  flex-shrink: 0;
  overflow: hidden;
}

.ap-strip-label {
  color: #888888;
}

.ap-strip-index {
  color: #aaaaaa;
  font-variant-numeric: tabular-nums;
}

.ap-strip-name {
  color: #ffd3a9;
  font-weight: 600;
  font-size: 12px;
}

.ap-strip-sep {
  color: #555555;
}

.ap-strip-meta {
  color: #aaaaaa;
}

.ap-tbl-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 36px;
  padding: 6px 12px;
  background: #1f1f1f;
  border-bottom: 1px solid #2a2a2a;
  flex-shrink: 0;
}

.ap-search {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 240px;
  padding: 4px 8px;
  border-radius: 3px;
  background: #161616;
  border: 1px solid #2a2a2a;
}

.ap-search input {
  flex: 1;
  background: transparent;
  outline: none;
  border: none;
  color: #cccccc;
  font-size: 11px;
  min-width: 0;
}

.ap-search input::placeholder {
  color: #666666;
}

.ap-count-chip {
  padding: 4px 8px;
  border-radius: 3px;
  background: #2a2a2a;
  color: #aaaaaa;
  font-size: 11px;
  flex-shrink: 0;
}

.ap-tbl-hdr,
.ap-tbl-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 10px;
  font-size: 11px;
  flex-shrink: 0;
}

.ap-tbl-hdr {
  height: 30px;
  background: #161616;
  border-bottom: 1px solid #2a2a2a;
  color: #dddddd;
  font-weight: 600;
}

.ap-tbl-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  background: #1a1a1a;
}

.ap-tbl-row {
  height: 28px;
  border-bottom: 1px solid #1a1a1a;
  cursor: pointer;
  color: #aaaaaa;
  border-left: 2px solid transparent;
  transition:
    background 0.1s ease,
    color 0.1s ease;
}

.ap-tbl-row:hover {
  background: #1d1d1d;
  color: #cccccc;
}

.ap-tbl-row[data-active] {
  background: #241d14;
  border-left-color: #ffad66;
}

.ap-tbl-row[data-active] .ap-col-id,
.ap-tbl-row[data-active] .ap-col-duration {
  color: #ffd3a9;
  font-weight: 600;
}

.ap-tbl-row[data-active] .ap-col-index {
  color: #ffad66;
  font-weight: 600;
}

.ap-tbl-row[data-active] .ap-col-size,
.ap-tbl-row[data-active] .ap-col-lang {
  color: #ffd3a9;
}

.ap-col-marker {
  width: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.ap-col-index {
  width: 48px;
  text-align: center;
  color: #888888;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.ap-col-id {
  flex: 1;
  min-width: 0;
  color: #cccccc;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ap-col-duration {
  width: 80px;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.ap-col-size {
  width: 80px;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.ap-col-lang {
  width: 64px;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.ap-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 32px;
  color: #888888;
  font-size: 12px;
}
</style>
