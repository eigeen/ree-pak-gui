<template>
  <v-dialog v-model="show" width="auto" max-height="600px" persistent>
    <v-card class="pa-2" max-width="600" prepend-icon="mdi-update" title="Update Available">
      <v-card-text>
        <div class="mb-4">
          <h6 class="text-h6 mb-2">Version v{{ updateStore.updateVersion?.version }}</h6>
          <p>Release Date: {{ updateStore.updateVersion?.pub_time }}</p>
          <p v-if="updateStore.updateVersion?.description">
            {{ updateStore.updateVersion?.description }}
          </p>
        </div>
        <div>
          <p v-if="!downloading">Will download and restart the application.</p>
          <v-progress-linear
            v-if="downloading"
            v-model="progress"
            class="mb-2"
            color="primary"
          ></v-progress-linear>
        </div>
      </v-card-text>

      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn v-if="!downloading" class="text-none" @click="show = false">Not Now</v-btn>
        <v-btn v-if="!downloading" class="text-none" color="primary" @click="startDownload"
          >Update</v-btn
        >
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
import { UpdateService } from '@/service/update'
import { useUpdateStore } from '@/store/update'
import { ShowError, ShowInfo } from '@/utils/message'
import { getCurrentWindow, ProgressBarStatus } from '@tauri-apps/api/window'
import { onMounted, ref } from 'vue'

const updateStore = useUpdateStore()

const show = ref(false)
const downloading = ref(false)
const progress = ref(0)

const startDownload = async () => {
  downloading.value = true
  try {
    const updateService = UpdateService.getInstance()

    const window = getCurrentWindow()
    await updateService.downloadUpdate(async (event) => {
      // handle progress
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
    ShowError(`Failed to download update: ${error}`)
    downloading.value = false
  }
}

onMounted(async () => {
  if (!updateStore.hasChecked) {
    try {
      const updateService = UpdateService.getInstance()
      updateStore.updateVersion = await updateService.checkForUpdates()
      updateStore.hasChecked = true
      if (updateStore.updateVersion) {
        ShowInfo('Update available. Click the button on the top right to download.')
      }
      console.debug('Update check complete.')
    } catch (err) {
      ShowError(`Failed to check for updates: ${err}`)
    }
  }
})

const popup = () => {
  show.value = true
}

defineExpose({ popup })
</script>
