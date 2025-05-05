<template>
  <div class="settings-switch">
    <div class="info">
      <span class="title">imtitle</span>
      <span class="description">imdescription</span>
    </div>
    <v-switch color="primary" hide-details></v-switch>
  </div>
</template>

<script setup lang="ts"></script>

<style scoped lang="scss">
.settings-switch {
  display: flex;
  flex-direction: row;
}

.info {
  flex: 1;
  display: flex;
  flex-direction: column;

  .title {
    font-size: 16px;
  }

  .description {
    font-size: 14px;
  }
}
</style>
