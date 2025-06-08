<template>
  <v-toolbar class="menu">
    <!-- Left Group -->
    <div class="slogan">REE PAK Tools</div>

    <v-btn-toggle class="mx-auto" mandatory density="compact" v-model="activeRoute" color="primary">
      <v-btn class="text-none" value="/unpack" text :to="{ name: 'UnpackView' }">Unpack</v-btn>
      <v-btn class="text-none" text disabled>Repack</v-btn>
    </v-btn-toggle>

    <!-- Right Group -->
    <div class="right-group">
      <div v-if="updateStore.updateVersion" class="red-dot">
        <v-btn icon="mdi-update" @click="showUpdateDialog"></v-btn>
      </div>
      <v-btn icon="mdi-github" @click="openUrl('https://github.com/eigeen/ree-pak-rs')"> </v-btn>
      <!-- <v-btn icon="mdi-cog" @click="openSettings"></v-btn> -->
    </div>

    <UpdateDialog ref="updateDialog"></UpdateDialog>
  </v-toolbar>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useUpdateStore } from '@/store/update'
import { useSettingsStore } from '@/store/settings'
import { openUrl } from '@tauri-apps/plugin-opener'

const updateStore = useUpdateStore()
const settingsStore = useSettingsStore()

const route = useRoute()
const activeRoute = ref(route.path)
const updateDialog = ref<any>(null)

watch(
  () => route.path,
  (newPath) => {
    activeRoute.value = newPath
  }
)

const showUpdateDialog = () => {
  updateDialog.value.popup()
}

const openSettings = () => {
  settingsStore.showSettings = true
}
</script>

<style scoped lang="scss">
.menu {
  background-color: transparent;
  padding: 0 16px;
}

.slogan {
  margin-right: 24px;
  font-weight: bold;
}

.right-group {
  display: flex;
  align-items: center;
}

.red-dot {
  position: relative;

  &::after {
    content: '';
    position: absolute;
    right: 12px;
    bottom: 12px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: red;
  }
}
</style>
