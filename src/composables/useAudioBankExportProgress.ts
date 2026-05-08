import { Channel } from '@tauri-apps/api/core'
import { join } from '@tauri-apps/api/path'
import { open as dialogOpen } from '@tauri-apps/plugin-dialog'
import { ref, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  audio_extract_wavs_with_progress,
  audio_terminate_extract,
  type AudioEntryInfo,
  type AudioExportProgressEvent,
  type AudioSourceRef
} from '@/api/tauri/pak'
import {
  ensureTaskProgressIdle,
  finishTaskProgress,
  tryStartTaskProgress,
  updateTaskProgress,
  useTaskProgressState
} from '@/service/taskProgress'
import { ShowError, ShowInfo, ShowWarn } from '@/utils/message'

type AudioExportOptions = {
  createBankDirectory?: boolean
}

type AudioBankExportProgressOptions = {
  source: Ref<AudioSourceRef | null>
  getBankDirectoryName: () => string
}

export function useAudioBankExportProgress(options: AudioBankExportProgressOptions) {
  const { t } = useI18n()
  const taskProgress = useTaskProgressState()
  const exporting = ref(false)

  async function exportEntries(entries: AudioEntryInfo[], exportOptions: AudioExportOptions = {}) {
    if (!ensureTaskProgressIdle(t('global.taskBusy'))) return
    if (!options.source.value || exporting.value || entries.length === 0) return

    const outputDir = await chooseAudioExportOutputDir(exportOptions)
    if (!outputDir) return

    await runAudioExport(entries, outputDir, options.source.value)
  }

  async function chooseAudioExportOutputDir(exportOptions: AudioExportOptions) {
    const target = await dialogOpen({
      directory: true,
      multiple: false,
      title: t('unpack.audioBankExportSelectDir')
    })
    if (typeof target !== 'string' || !target) return
    return await resolveExportOutputDir(target, exportOptions)
  }

  async function resolveExportOutputDir(target: string, exportOptions: AudioExportOptions) {
    if (!exportOptions.createBankDirectory) return target
    return await join(target, options.getBankDirectoryName())
  }

  async function runAudioExport(
    entries: AudioEntryInfo[],
    outputDir: string,
    exportSource: AudioSourceRef
  ) {
    exporting.value = true
    const taskId = startAudioExportTask()
    if (!taskId) {
      exporting.value = false
      return
    }

    const onEvent = new Channel<AudioExportProgressEvent>()
    bindAudioExportProgress(onEvent, taskId)

    try {
      const paths = await audio_extract_wavs_with_progress(
        {
          source: exportSource,
          indices: entries.map((entry) => entry.index),
          outputDir
        },
        onEvent
      )
      finishAudioExport(taskId, paths.length)
      ShowInfo(t('unpack.audioBankExportDone', { count: paths.length }))
    } catch (error) {
      failAudioExport(taskId, error)
      ShowError(formatAudioExportError(error))
    } finally {
      exporting.value = false
    }
  }

  function startAudioExportTask() {
    return tryStartTaskProgress({
      taskId: 'audio-export',
      title: t('unpack.exportingAudio'),
      progressLabel: t('unpack.exporting'),
      runningDescription: t('unpack.processing'),
      successDescription: t('unpack.done'),
      terminatedDescription: t('unpack.taskStopped'),
      closeLabel: t('unpack.close'),
      terminateLabel: t('unpack.terminate'),
      confirmTitle: t('unpack.confirmTermination'),
      confirmDescription: t('unpack.confirmTerminationText'),
      busyMessage: t('global.taskBusy'),
      onTerminate: async () => {
        await audio_terminate_extract()
        ShowWarn(t('unpack.taskStopped'))
      }
    })
  }

  function bindAudioExportProgress(onEvent: Channel<AudioExportProgressEvent>, taskId: string) {
    onEvent.onmessage = (event) => {
      if (event.event === 'workStart') {
        updateTaskProgress(taskId, {
          totalFileCount: event.data.count,
          finishFileCount: 0,
          currentFile: '',
          description: t('unpack.processing')
        })
        return
      }

      if (event.event === 'fileDone') {
        updateTaskProgress(taskId, {
          finishFileCount: event.data.finishCount,
          currentFile: event.data.path
        })
        return
      }

      if (event.event === 'workFinished') {
        finishAudioExport(taskId, taskProgress.totalFileCount)
        return
      }

      if (event.event === 'error') {
        failAudioExport(taskId, event.data.error)
      }
    }
  }

  function finishAudioExport(taskId: string, count: number) {
    finishTaskProgress(taskId, {
      status: 'success',
      finishFileCount: count,
      totalFileCount: taskProgress.totalFileCount || count
    })
  }

  function failAudioExport(taskId: string, error: unknown) {
    const message = error instanceof Error ? error.message : String(error)
    finishTaskProgress(taskId, {
      status: 'error',
      errorMessage: message,
      currentFile: message,
      description: t('unpack.taskStopped')
    })
  }

  function formatAudioExportError(error: unknown) {
    const message = error instanceof Error ? error.message : String(error)
    return `${t('unpack.audioBankExportFailed')}: ${message}`
  }

  return {
    exporting,
    exportEntries
  }
}
