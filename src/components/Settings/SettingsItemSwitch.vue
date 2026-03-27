<template>
  <SettingsItem :title="title" :description="description">
    <Switch v-model="value" />
  </SettingsItem>
</template>

<script setup lang="ts">
import { Switch } from '@/components/ui/switch'

interface Props {
  title: string
  description: string
}

const value = defineModel<boolean>({ default: false })

defineProps<Props>()
</script>
