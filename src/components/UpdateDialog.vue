<template>
  <v-dialog v-model="show" width="auto" max-height="600px" persistent>
    <v-card class="pa-2" max-width="600" prepend-icon="mdi-update" title="Update Available">
      <v-card-text>
        <div class="mb-4">
          <h6 class="text-h6 mb-2">Version v{{ updateStore.updateVersion?.version }}</h6>
          <p>Release Date: {{ updateStore.updateVersion?.pub_time }}</p>
          <p>Description: This is a major update with many new features and improvements.</p>
        </div>
      </v-card-text>

      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn class="text-none" @click="show = false">Not Now</v-btn>
        <v-btn v-if="!downloading" class="text-none" color="primary" @click="startDownload"
          >Download</v-btn
        >
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts" setup>
import { Update } from '@/api/tauri/update'
import { useUpdateStore } from '@/store/update'
import { ShowError, ShowInfo } from '@/utils'
import { onMounted, ref } from 'vue'

const updateStore = useUpdateStore()

const show = ref(false)
const downloading = ref(false)

const startDownload = async () => {
  downloading.value = true
  show.value = false
  try {
    await Update.perform(updateStore.updateVersion!)
    ShowInfo('Update applied successfully. Restart the application to complete the update.')
  } catch (error) {
    ShowError(`Failed to download update: ${error}`)
    downloading.value = false
  }
}

onMounted(async () => {
  if (updateStore.hasChecked) return

  try {
    updateStore.updateVersion = await Update.check()
    console.log('UpdateVersion', updateStore.updateVersion)
    ShowInfo('Update available. Click the button on the top right to download.')
  } catch (error) {
  } finally {
    updateStore.hasChecked = true
  }
})

const popup = () => {
  show.value = true
}

defineExpose({ popup })
</script>
