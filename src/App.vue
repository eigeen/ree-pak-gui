<template>
  <v-app>
    <v-main class="root">
      <header>
        <Menu></Menu>
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

<style lang="scss">
html {
  /* disable vuetify scrollbar */
  overflow-y: auto !important;
}

html::-webkit-scrollbar {
  display: none;
}

.el-menu--horizontal {
  --el-menu-horizontal-height: 50px;
}

.root {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: #f5f5f5;
}

el-menu {
  .right-group {
    display: flex;
    align-items: center;
    margin-left: auto;
  }
}

.content {
  margin: 0 10px;
  flex: 1;
}
</style>
