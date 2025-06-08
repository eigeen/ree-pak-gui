<template>
  <div class="hover-bubble">
    <v-icon icon="mdi-help-circle" size="16" color="#1976d2" />
    <v-tooltip activator="parent" location="top">
      <slot />
    </v-tooltip>
  </div>
</template>

<script setup lang="ts"></script>

<style scoped lang="scss">
.hover-bubble {
  display: inline-flex;
  align-items: center;
  position: relative;
  border-radius: 50%;
  // background-color: #1976d2;
  height: 16px;
  width: 16px;

  // transition: background-color 0.2s;
  // &:hover {
  //   background-color: #1565c0;
  // }
}
</style>
