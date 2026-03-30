<template>
  <Transition
    enter-active-class="transition duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]"
    enter-from-class="translate-y-8 opacity-0"
    enter-to-class="translate-y-0 opacity-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="translate-y-0 opacity-100"
    leave-to-class="translate-y-8 opacity-0"
  >
    <div
      v-if="taskProgress.visible"
      class="pointer-events-none fixed inset-x-0 bottom-5 z-50 flex justify-center px-4"
    >
      <section
        class="pointer-events-auto w-full max-w-md rounded-2xl border border-border/80 bg-background/94 shadow-2xl shadow-black/20 backdrop-blur-md"
      >
        <div class="flex items-start gap-3 px-4 py-3">
          <div class="min-w-0 flex-1 space-y-3">
            <div class="space-y-1">
              <div class="flex items-center justify-between gap-3">
                <p class="truncate text-sm font-semibold text-foreground">
                  {{ taskProgress.title }}
                </p>
                <p class="shrink-0 text-xs text-muted-foreground">
                  {{ taskProgress.finishFileCount }} / {{ taskProgress.totalFileCount }}
                </p>
              </div>
              <p class="truncate text-xs text-muted-foreground">{{ taskProgress.description }}</p>
            </div>

            <Progress :model-value="progressValue" class="h-1.5 rounded-full" />

            <div class="flex items-center justify-between gap-3">
              <p class="truncate text-xs text-muted-foreground">
                <span class="text-foreground/80">{{ taskProgress.progressLabel }}</span>
                <span class="ml-1">{{ taskProgress.currentFile || taskProgress.placeholder }}</span>
              </p>
              <Button
                size="sm"
                :variant="taskProgress.working ? 'destructive' : 'outline'"
                class="h-8 shrink-0 px-3"
                @click="handleAction"
              >
                {{ taskProgress.working ? taskProgress.terminateLabel : taskProgress.closeLabel }}
              </Button>
            </div>
          </div>
        </div>
      </section>
    </div>
  </Transition>

  <AlertDialog v-model:open="showConfirmTermination">
    <AlertDialogContent class="rounded-[1rem] border-border/80 bg-background/96">
      <AlertDialogHeader>
        <AlertDialogTitle>{{ taskProgress.confirmTitle }}</AlertDialogTitle>
        <AlertDialogDescription>{{ taskProgress.confirmDescription }}</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>{{ taskProgress.closeLabel }}</AlertDialogCancel>
        <AlertDialogAction @click="handleConfirmTermination">
          {{ taskProgress.terminateLabel }}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { ProgressBarStatus, getCurrentWindow } from '@tauri-apps/api/window'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import {
  closeTaskProgressPanel,
  resetTaskProgress,
  terminateTaskProgress,
  useTaskProgressState
} from '@/service/taskProgress'

const taskProgress = useTaskProgressState()
const showConfirmTermination = ref(false)
const progressValue = computed(() =>
  taskProgress.totalFileCount <= 0
    ? 0
    : (taskProgress.finishFileCount / taskProgress.totalFileCount) * 100
)

let hideTimer: ReturnType<typeof window.setTimeout> | undefined
const AUTO_HIDE_DELAY = 2800

watch(
  () => [taskProgress.visible, taskProgress.working, taskProgress.taskId] as const,
  ([open, working, taskId]) => {
    clearHideTimer()
    if (!open || working || !taskId) return

    hideTimer = window.setTimeout(() => {
      closeTaskProgressPanel()
      resetTaskProgress(taskId)
    }, AUTO_HIDE_DELAY)
  },
  { immediate: true }
)

watch(
  () => [taskProgress.visible, taskProgress.working, progressValue.value] as const,
  async ([open, working, progress]) => {
    const appWindow = getCurrentWindow()
    if (!open) {
      await appWindow.setProgressBar({ status: ProgressBarStatus.None, progress: 0 })
      return
    }

    if (working) {
      await appWindow.setProgressBar({
        status: ProgressBarStatus.Normal,
        progress: Math.max(0, Math.min(100, Math.floor(progress)))
      })
      return
    }

    await appWindow.setProgressBar({ status: ProgressBarStatus.None, progress: 0 })
  },
  { immediate: true }
)

onBeforeUnmount(async () => {
  clearHideTimer()
  await getCurrentWindow().setProgressBar({ status: ProgressBarStatus.None, progress: 0 })
})

function clearHideTimer() {
  if (!hideTimer) return
  window.clearTimeout(hideTimer)
  hideTimer = undefined
}

function handleAction() {
  if (taskProgress.working) {
    showConfirmTermination.value = true
    return
  }

  closeTaskProgressPanel()
  resetTaskProgress(taskProgress.taskId)
}

async function handleConfirmTermination() {
  showConfirmTermination.value = false
  await terminateTaskProgress()
}
</script>
