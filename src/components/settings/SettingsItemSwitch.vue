<template>
  <SettingsItem :title="title" :description="description">
    <v-switch
      class="mr-2"
      hide-details
      density="compact"
      color="primary"
      v-model="value"
    ></v-switch>
  </SettingsItem>
</template>

<script setup lang="ts">
import SettingsItem from '@/components/settings/SettingsItem.vue'

interface Props {
  title: string
  description: string
}

const value = defineModel<boolean>()
defineProps<Props>()
</script>

<style scoped lang="scss"></style>
