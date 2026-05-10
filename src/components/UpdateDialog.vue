<template>
  <Dialog v-model:open="show">
    <DialogContent
      :show-close-button="!downloading"
      class="max-h-[90vh] max-w-xl grid-rows-[auto_minmax(0,1fr)_auto] rounded-3xl border-white/60 bg-background/96"
    >
      <DialogHeader class="space-y-3">
        <div class="flex items-center gap-3">
          <Download class="size-5" />
          <div>
            <DialogTitle class="text-base font-bold leading-[1.35] tracking-normal">{{
              t('updateDialog.updateAvailable')
            }}</DialogTitle>
            <DialogDescription class="mt-1 text-sm leading-[1.4]">{{
              t('updateDialog.description')
            }}</DialogDescription>
          </div>
        </div>
      </DialogHeader>

      <div class="flex min-h-0 flex-col gap-5 overflow-hidden px-2">
        <h3 class="text-base font-semibold">
          {{ t('updateDialog.version') }} v{{ updateState.updateVersion?.version }}
        </h3>
        <p class="text-sm text-muted-foreground">
          {{ t('updateDialog.releaseDate') }}: {{ updateState.updateVersion?.pub_time }}
        </p>

        <ScrollArea v-if="renderedDescription" class="min-h-0 flex-1 pr-3">
          <div
            ref="descriptionRef"
            class="update-description-markdown break-words text-[0.8125rem] leading-[1.65] [&>*:first-child]:mt-0 [&>*:last-child]:mb-0"
            v-html="renderedDescription"
            @click="handleDescriptionClick"
          />
        </ScrollArea>

        <div class="space-y-3">
          <p v-if="!downloading" class="text-sm text-muted-foreground">
            {{ t('updateDialog.willDownloadAndRestart') }}
          </p>
          <div v-else class="space-y-2">
            <Progress :model-value="progress" class="h-2.5 rounded-full" />
            <p class="text-sm text-muted-foreground">{{ progress }}%</p>
          </div>
        </div>
      </div>

      <DialogFooter class="gap-2">
        <Button v-if="!downloading" variant="outline" @click="show = false">
          {{ t('updateDialog.notNow') }}
        </Button>
        <Button v-if="!downloading" @click="startDownload">
          {{ t('updateDialog.update') }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script lang="ts" setup>
import MarkdownIt from 'markdown-it/dist/index.cjs.js'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getCurrentWindow, ProgressBarStatus } from '@tauri-apps/api/window'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Download } from 'lucide-vue-next'
import { UpdateService } from '@/service/update'
import { useUpdateStore } from '@/store/update'
import { logFrontendInfo } from '@/utils/frontendLog'
import { ShowError, ShowInfo } from '@/utils/message'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog'
import { Progress } from '@/components/ui/progress'
import { ScrollArea } from '@/components/ui/scroll-area'

const { t } = useI18n()
const updateStore = useUpdateStore()
const updateState = updateStore as any

const show = ref(false)
const downloading = ref(false)
const progress = ref(0)
const descriptionRef = ref<HTMLElement | null>(null)
const markdown = new MarkdownIt({
  html: false,
  breaks: true,
  linkify: true
})
const renderedDescription = computed(() => {
  const description = updateState.updateVersion?.description
  return description ? markdown.render(description) : ''
})

const handleDescriptionClick = async (event: MouseEvent) => {
  const container = descriptionRef.value
  const target = event.target
  if (!container || !(target instanceof Element)) {
    return
  }

  const anchor = target.closest('a')
  if (!(anchor instanceof HTMLAnchorElement) || !container.contains(anchor)) {
    return
  }

  const href = anchor.getAttribute('href')
  if (!href) {
    return
  }

  let protocol = ''
  try {
    protocol = new URL(href).protocol
  } catch {
    return
  }

  if (!['http:', 'https:', 'mailto:'].includes(protocol)) {
    return
  }

  event.preventDefault()
  await openUrl(href)
}

const startDownload = async () => {
  downloading.value = true
  logFrontendInfo('update.dialog', 'user confirmed download')
  try {
    const updateService = UpdateService.getInstance()
    const window = getCurrentWindow()

    await updateService.downloadUpdate(async (event) => {
      if (event.type === 'loadstart') {
        await window.setProgressBar({
          status: ProgressBarStatus.Normal,
          progress: 0
        })
      } else if (event.type === 'load') {
        progress.value = Math.floor((event.loaded / event.total) * 100)
        await window.setProgressBar({
          status: ProgressBarStatus.Normal,
          progress: progress.value
        })
      } else if (event.type === 'loadend') {
        await window.setProgressBar({
          status: ProgressBarStatus.None,
          progress: 0
        })
        downloading.value = false
      }
    })

    await updateService.performUpdate()
  } catch (error) {
    ShowError(t('global.failedDownloadUpdate', { error: String(error) }))
    downloading.value = false
  }
}

onMounted(async () => {
  if (!updateState.hasChecked) {
    try {
      const updateService = UpdateService.getInstance()
      updateState.updateVersion = await updateService.checkForUpdates()
      updateState.hasChecked = true
      if (updateState.updateVersion) {
        ShowInfo(t('global.updateAvailable'))
        logFrontendInfo(
          'update.dialog',
          `update ready version=${updateState.updateVersion.version}`
        )
      }
    } catch (err) {
      ShowError(t('global.failedCheckUpdate', { error: String(err) }))
    }
  }
})

const popup = () => {
  show.value = true
}

defineExpose({ popup })
</script>

<style scoped>
.update-description-markdown :deep(p),
.update-description-markdown :deep(ul),
.update-description-markdown :deep(ol),
.update-description-markdown :deep(blockquote),
.update-description-markdown :deep(pre),
.update-description-markdown :deep(h1),
.update-description-markdown :deep(h2),
.update-description-markdown :deep(h3),
.update-description-markdown :deep(h4),
.update-description-markdown :deep(h5),
.update-description-markdown :deep(h6) {
  margin: 0.625rem 0;
}

.update-description-markdown :deep(ul),
.update-description-markdown :deep(ol) {
  padding-left: 1.25rem;
}

/* 标题样式 */
.update-description-markdown :deep(h1),
.update-description-markdown :deep(h2),
.update-description-markdown :deep(h3),
.update-description-markdown :deep(h4),
.update-description-markdown :deep(h5),
.update-description-markdown :deep(h6) {
  margin-top: 24px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.25;
}

/* 分割线 */
.update-description-markdown :deep(h1) {
  font-size: 2em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid #d0d7de7a;
}
.update-description-markdown :deep(h2) {
  font-size: 1.5em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid #d0d7de7a;
}

.update-description-markdown :deep(h3) {
  font-size: 1.25em;
}
.update-description-markdown :deep(h4) {
  font-size: 1em;
}
.update-description-markdown :deep(h5) {
  font-size: 0.875em;
}
.update-description-markdown :deep(h6) {
  font-size: 0.85em;
  color: #57606a;
}

/* 段落与间距 */
.update-description-markdown :deep(p),
.update-description-markdown :deep(blockquote),
.update-description-markdown :deep(ul),
.update-description-markdown :deep(ol),
.update-description-markdown :deep(dl),
.update-description-markdown :deep(table),
.update-description-markdown :deep(pre),
.update-description-markdown :deep(details) {
  margin-top: 0;
  margin-bottom: 16px;
}

.update-description-markdown :deep(li + li) {
  margin-top: 0.25rem;
}

.update-description-markdown :deep(a) {
  color: var(--color-primary);
  text-decoration: underline;
  text-underline-offset: 0.18em;
}

.update-description-markdown :deep(code) {
  border: 1px solid color-mix(in srgb, var(--color-border) 88%, transparent);
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--color-secondary) 72%, transparent);
  padding: 0.1rem 0.35rem;
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.update-description-markdown :deep(pre) {
  overflow-x: auto;
  border: 1px solid color-mix(in srgb, var(--color-border) 88%, transparent);
  border-radius: 0.75rem;
  background: color-mix(in srgb, var(--color-secondary) 72%, transparent);
  padding: 0.75rem 0.875rem;
}

.update-description-markdown :deep(pre code) {
  border: 0;
  background: transparent;
  padding: 0;
}

.update-description-markdown :deep(blockquote) {
  border-left: 3px solid color-mix(in srgb, var(--color-primary) 40%, transparent);
  padding-left: 0.875rem;
  color: color-mix(in srgb, var(--color-muted-foreground) 88%, white);
}
</style>
