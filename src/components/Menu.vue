<template>
  <v-toolbar class="menu">
    <!-- Left Group -->
    <div class="left-group">
      <div class="slogan">{{ t('menu.slogan') }}</div>
    </div>

    <!-- Center Group -->
    <div class="center-group">
      <v-btn-toggle mandatory density="compact" v-model="activeRoute" color="primary">
        <v-btn class="text-none" value="/unpack" text :to="{ name: 'UnpackView' }">{{ t('menu.unpack') }}</v-btn>
        <v-btn class="text-none" value="/pack" text :to="{ name: 'PackView' }">{{ t('menu.repack') }}</v-btn>
      </v-btn-toggle>
    </div>

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
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
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
  display: flex;
  align-items: center;
}

.left-group {
  flex: 1;
  display: flex;
  justify-content: flex-start;
}

.slogan {
  font-weight: bold;
}

.center-group {
  flex: 0 0 auto;
  display: flex;
  justify-content: center;
}

.right-group {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: flex-end;
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
