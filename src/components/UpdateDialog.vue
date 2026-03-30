<template>
  <Dialog v-model:open="show">
    <DialogContent
      :show-close-button="!downloading"
      class="max-w-xl rounded-[1.5rem] border-white/60 bg-background/96"
    >
      <DialogHeader class="space-y-3">
        <div class="flex items-center gap-3">
          <div
            class="flex size-11 items-center justify-center rounded-2xl border border-primary/20 bg-primary/10 text-primary"
          >
            <Download class="size-5" />
          </div>
          <div>
            <DialogTitle>{{ t('updateDialog.updateAvailable') }}</DialogTitle>
            <DialogDescription>{{ t('updateDialog.description') }}</DialogDescription>
          </div>
        </div>
      </DialogHeader>

      <div class="space-y-5">
        <div class="app-panel-muted space-y-2 p-4">
          <h3 class="text-base font-semibold">
            {{ t('updateDialog.version') }} v{{ updateState.updateVersion?.version }}
          </h3>
          <p class="text-sm text-muted-foreground">
            {{ t('updateDialog.releaseDate') }}: {{ updateState.updateVersion?.pub_time }}
          </p>
          <p
            v-if="updateState.updateVersion?.description"
            class="text-sm leading-6 text-muted-foreground"
          >
            {{ updateState.updateVersion?.description }}
          </p>
        </div>

        <div class="space-y-3">
          <p v-if="!downloading" class="text-sm text-muted-foreground">
            {{ t('updateDialog.willDownloadAndRestart') }}
          </p>
          <div v-else class="space-y-2">
            <Progress :model-value="progress" class="h-2.5 rounded-full" />
            <p class="text-sm text-muted-foreground">{{ progress }}%</p>
          </div>
        </div>
      </div>

      <DialogFooter class="gap-2">
        <Button v-if="!downloading" variant="outline" @click="show = false">
          {{ t('updateDialog.notNow') }}
        </Button>
        <Button v-if="!downloading" @click="startDownload">
          {{ t('updateDialog.update') }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script lang="ts" setup>
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getCurrentWindow, ProgressBarStatus } from '@tauri-apps/api/window'
import { Download } from 'lucide-vue-next'
import { UpdateService } from '@/service/update'
import { useUpdateStore } from '@/store/update'
import { ShowError, ShowInfo } from '@/utils/message'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog'
import { Progress } from '@/components/ui/progress'

const { t } = useI18n()
const updateStore = useUpdateStore()
const updateState = updateStore as any

const show = ref(false)
const downloading = ref(false)
const progress = ref(0)

const startDownload = async () => {
  downloading.value = true
  try {
    const updateService = UpdateService.getInstance()
    const window = getCurrentWindow()

    await updateService.downloadUpdate(async (event) => {
      if (event.type === 'loadstart') {
        await window.setProgressBar({
          status: ProgressBarStatus.Normal,
          progress: 0
        })
      } else if (event.type === 'load') {
        progress.value = Math.floor((event.loaded / event.total) * 100)
        await window.setProgressBar({
          status: ProgressBarStatus.Normal,
          progress: progress.value
        })
      } else if (event.type === 'loadend') {
        await window.setProgressBar({
          status: ProgressBarStatus.None,
          progress: 0
        })
        downloading.value = false
      }
    })

    await updateService.performUpdate()
  } catch (error) {
    ShowError(t('global.failedDownloadUpdate', { error: String(error) }))
    downloading.value = false
  }
}

onMounted(async () => {
  if (!updateState.hasChecked) {
    try {
      const updateService = UpdateService.getInstance()
      updateState.updateVersion = await updateService.checkForUpdates()
      updateState.hasChecked = true
      if (updateState.updateVersion) {
        ShowInfo(t('global.updateAvailable'))
      }
      console.debug('Update check complete.')
    } catch (err) {
      ShowError(t('global.failedCheckUpdate', { error: String(err) }))
    }
  }
})

const popup = () => {
  show.value = true
}

defineExpose({ popup })
</script>
