<template>
  <v-navigation-drawer v-model="settingsStore.showSettings" temporary location="right" width="300">
    <v-toolbar density="comfortable" title="Settings">
      <template v-slot:append>
        <div class="d-flex ga-1">
          <v-btn icon="mdi-close" @click="settingsStore.showSettings = false"></v-btn>
        </div>
      </template>
    </v-toolbar>

    <div class="settings-list">
        <div>1</div>
        <div>2</div>
        <div>3</div>
        <SettingsSwitch></SettingsSwitch>
        <v-divider></v-divider>
    </div>
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { useSettingsStore } from '@/store/settings'

const settingsStore = useSettingsStore()
</script>

<style scoped lang="scss">
.settings-list {
  display: flex;
  flex-direction: column;
  margin: 0 8px
}
</style>
