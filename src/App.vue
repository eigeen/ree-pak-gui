<template>
  <v-app>
    <v-main class="root">
      <header>
        <el-menu default-active="/unpack" mode="horizontal" class="menu" router>
          <el-menu-item index="/unpack">Unpack</el-menu-item>
          <el-menu-item index="/pack" disabled>Pack</el-menu-item>
        </el-menu>
      </header>
      <div class="content">
        <router-view />
      </div>
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { RouterView } from 'vue-router'
</script>

<style>
html {
  /* disable vuetify scrollbar */
  overflow-y: auto !important;
}

.root {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.content {
  margin: 0 10px;
  flex: 1;
}
</style>
