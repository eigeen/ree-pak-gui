<template>
  <div class="space-y-4">
    <div class="flex flex-wrap items-center gap-2">
      <Badge :variant="status?.installed ? 'default' : 'outline'">
        {{ status?.installed ? t('settings.extensionInstalled') : t('settings.extensionMissing') }}
      </Badge>
      <span class="text-xs text-muted-foreground">
        {{ platformLabel }}
      </span>
    </div>

    <div class="rounded-md border border-border/70 bg-secondary/25 px-3 py-2 text-xs">
      <p class="break-all text-muted-foreground">
        {{ t('settings.vgmstreamExecutablePath') }}:
        <span class="text-foreground">{{ displayPath }}</span>
      </p>
      <p v-if="status?.assetName" class="mt-1 text-muted-foreground">
        {{ t('settings.vgmstreamAssetName') }}:
        <span class="text-foreground">{{ status.assetName }}</span>
      </p>
    </div>

    <div v-if="installing" class="space-y-2">
      <Progress :model-value="progress" class="h-2 rounded-full" />
      <p class="text-xs text-muted-foreground">{{ progress }}%</p>
    </div>

    <p v-if="lastInstallLabel" class="text-sm text-muted-foreground">
      {{ lastInstallLabel }}
    </p>

    <p v-if="errorMessage" class="text-sm text-destructive">
      {{ errorMessage }}
    </p>

    <p class="text-sm text-muted-foreground">
      {{ t('settings.vgmstreamDynamicLibraryHint') }}
    </p>

    <div class="flex flex-wrap gap-2">
      <Button variant="outline" size="sm" :disabled="loading || installing" @click="refreshStatus">
        <RefreshCw class="size-4" :class="{ 'animate-spin': loading }" />
        {{ t('settings.refresh') }}
      </Button>
      <Button
        variant="outline"
        size="sm"
        :disabled="!status?.installDir || installing"
        @click="openInstallDir"
      >
        <FolderOpen class="size-4" />
        {{ t('settings.openInstallDir') }}
      </Button>
      <Button variant="outline" size="sm" @click="openReleases">
        <ExternalLink class="size-4" />
        {{ t('settings.openReleases') }}
      </Button>
      <Button size="sm" :disabled="!canInstall" @click="install">
        <LoaderCircle v-if="installing" class="size-4 animate-spin" />
        <Download v-else class="size-4" />
        {{ status?.installed ? t('settings.reinstallExtension') : t('settings.installExtension') }}
      </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { openPath, openUrl } from '@tauri-apps/plugin-opener'
import { Download, ExternalLink, FolderOpen, LoaderCircle, RefreshCw } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { VGMSTREAM_RELEASES_URL } from '@/api/http/vgmstream'
import type { VgmstreamStatus } from '@/api/tauri/utils'
import { VgmstreamService } from '@/service/vgmstream'
import { ShowError, ShowInfo } from '@/utils/message'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'

const { t } = useI18n()
const service = VgmstreamService.getInstance()

const status = ref<VgmstreamStatus | null>(null)
const loading = ref(false)
const installing = ref(false)
const progress = ref(0)
const errorMessage = ref('')
const lastInstallLabel = ref('')

const platformLabel = computed(() => {
  if (!status.value) return t('settings.extensionStatusLoading')
  return `${status.value.platform}-${status.value.arch}`
})

const displayPath = computed(() => {
  return status.value?.executablePath ?? status.value?.expectedPath ?? t('settings.unknown')
})

const canInstall = computed(() => {
  return !!status.value?.assetName && !loading.value && !installing.value
})

async function refreshStatus() {
  loading.value = true
  errorMessage.value = ''

  try {
    status.value = await service.getStatus()
  } catch (error) {
    errorMessage.value = String(error)
    ShowError(error)
  } finally {
    loading.value = false
  }
}

async function install() {
  if (!canInstall.value) return

  installing.value = true
  progress.value = 0
  errorMessage.value = ''
  lastInstallLabel.value = ''

  try {
    const result = await service.downloadAndInstall(async (event) => {
      if (event.type === 'loadstart') {
        progress.value = 0
        return
      }

      if (event.type === 'load') {
        progress.value = Math.floor((event.loaded / event.total) * 100)
        return
      }

      if (event.type === 'loadend') {
        progress.value = 100
      }
    })

    status.value = result.status
    lastInstallLabel.value = t('settings.vgmstreamInstallSource', {
      tag: result.release.tagName,
      source:
        result.source === 'latest'
          ? t('settings.vgmstreamSourceLatest')
          : t('settings.vgmstreamSourceFallback')
    })
    ShowInfo(t('settings.extensionInstallDone'))
  } catch (error) {
    errorMessage.value = String(error)
    ShowError(t('settings.extensionInstallFailed', { error: String(error) }))
  } finally {
    installing.value = false
  }
}

async function openInstallDir() {
  if (status.value?.installDir) {
    await openPath(status.value.installDir)
  }
}

async function openReleases() {
  await openUrl(VGMSTREAM_RELEASES_URL)
}

onMounted(refreshStatus)
</script>
