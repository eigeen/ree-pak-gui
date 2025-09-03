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
      
      <!-- 工具菜单 -->
      <v-menu offset-y>
        <template v-slot:activator="{ props }">
          <v-btn class="text-none ml-2" text v-bind="props">
            工具
            <v-icon right>mdi-chevron-down</v-icon>
          </v-btn>
        </template>
        <v-list>
          <v-list-item 
            v-for="tool in availableTools" 
            :key="tool.id"
            :to="`/tools/${tool.id}`"
          >
            <template v-slot:prepend>
              <v-icon v-if="tool.icon" :icon="tool.icon"></v-icon>
            </template>
            <v-list-item-title>{{ tool.title }}</v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>
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
import { getAllTools } from '@/config/tools'

const { t } = useI18n()
const updateStore = useUpdateStore()
const settingsStore = useSettingsStore()

const route = useRoute()
const activeRoute = ref(route.path)
const updateDialog = ref<any>(null)

// 获取所有可用工具
const availableTools = getAllTools()

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
