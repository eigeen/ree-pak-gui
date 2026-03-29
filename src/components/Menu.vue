<template>
  <header class="desktop-header">
    <div class="desktop-menubar">
      <div class="flex min-w-0 items-center gap-5">
        <div class="desktop-brand">
          <PackageOpen class="size-4" />
          <span>REE Pak Tool</span>
        </div>

        <div class="desktop-command-nav">
          <RouterLink :class="topNavClass('/unpack')" :to="{ name: 'UnpackView' }">
            {{ t('menu.unpack') }}
          </RouterLink>
          <RouterLink :class="topNavClass('/pack')" :to="{ name: 'PackView' }">
            {{ t('menu.repack') }}
          </RouterLink>
          <RouterLink :class="topNavClass('/settings')" :to="{ name: 'SettingsView' }">
            {{ t('menu.settings') }}
          </RouterLink>
        </div>
      </div>

      <div class="desktop-topbar-right">
        <Button
          v-if="updateStore.updateVersion"
          variant="outline"
          size="sm"
          class="desktop-command-button relative"
          @click="showUpdateDialog"
        >
          <Download class="size-4" />
          <span>{{ t('updateDialog.updateAvailable') }}</span>
          <span class="absolute right-2 top-1.5 size-1.5 rounded-full bg-destructive" />
        </Button>

        <Button
          variant="ghost"
          size="icon-sm"
          class="desktop-icon-button"
          @click="openUrl('https://github.com/eigeen/ree-pak-rs')"
        >
          <Github class="size-4" />
        </Button>
      </div>
    </div>

    <div v-if="!isSettingsView" class="desktop-window-toolbar">
      <div class="flex min-w-0 items-center gap-0">
        <DesktopMenuBar class="min-w-0" :items="desktopMenuItems" />
      </div>

      <div class="flex-1" />
    </div>

    <FileNameTable
      ref="fileNameTable"
      v-model="unpackState.fileList"
      :show-manage-button="false"
      :show-selector="false"
      class="hidden"
    />
    <UpdateDialog ref="updateDialog" />
  </header>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { openUrl } from '@tauri-apps/plugin-opener'
import {
  Download,
  FolderOpen,
  FolderPlus,
  Github,
  PackageOpen,
  PackagePlus,
  Play,
  RefreshCw,
  Settings,
  Trash2,
  Wrench
} from 'lucide-vue-next'
import { cn } from '@/lib/utils'
import { getAllTools } from '@/config/tools'
import DesktopMenuBar from '@/components/DesktopMenuBar.vue'
import FileNameTable from '@/components/FileNameTable/FileNameTable.vue'
import { useFileListStore } from '@/store/filelist'
import { useUpdateStore } from '@/store/update'
import { useWorkStore } from '@/store/work'
import { Button, buttonVariants } from '@/components/ui/button'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const fileListStore = useFileListStore()
const updateStore = useUpdateStore()
const workStore = useWorkStore()

const updateDialog = ref<{ popup: () => void } | null>(null)
const fileNameTable = ref<{ openManager: () => void } | null>(null)
const availableTools = getAllTools()

const isUnpackView = computed(() => route.name === 'UnpackView')
const isPackView = computed(() => route.name === 'PackView')
const isSettingsView = computed(() => route.name === 'SettingsView')
const unpackState = workStore.unpack
const topNavClass = (path: string) =>
  cn(
    buttonVariants({ variant: route.path === path ? 'secondary' : 'ghost', size: 'sm' }),
    'desktop-command-button'
  )

function dispatchUnpackAction(action: 'open-paks' | 'render-tree') {
  window.dispatchEvent(new CustomEvent(`unpack:${action}`))
}

function dispatchPackAction(
  action: 'add-folder' | 'add-pak' | 'clear-files' | 'select-export-directory' | 'export'
) {
  window.dispatchEvent(new CustomEvent(`pack:${action}`))
}

function showUpdateDialog() {
  updateDialog.value?.popup()
}

function openPathListManager() {
  fileNameTable.value?.openManager()
}

const desktopMenuItems = computed(() => {
  const items: Array<{
    key: string
    label: string
    items: Array<{
      key: string
      label: string
      icon?: unknown
      destructive?: boolean
      action: () => void
    }>
  }> = []

  if (isUnpackView.value) {
    items.push(
      {
        key: 'resources',
        label: t('menu.resources'),
        items: [
          {
            key: 'manage-path-lists',
            label: t('menu.managePathLists'),
            icon: Wrench,
            action: openPathListManager
          },
          {
            key: 'open-paks',
            label: t('menu.openPaks'),
            icon: FolderOpen,
            action: () => dispatchUnpackAction('open-paks')
          }
        ]
      },
      {
        key: 'actions',
        label: t('menu.actions'),
        items: [
          {
            key: 'render-tree',
            label: t('menu.reloadTree'),
            icon: RefreshCw,
            action: () => dispatchUnpackAction('render-tree')
          }
        ]
      }
    )
  } else if (isPackView.value) {
    items.push(
      {
        key: 'resources',
        label: t('menu.resources'),
        items: [
          {
            key: 'add-folder',
            label: t('pack.addFolder'),
            icon: FolderPlus,
            action: () => dispatchPackAction('add-folder')
          },
          {
            key: 'add-pak',
            label: t('pack.addPak'),
            icon: PackagePlus,
            action: () => dispatchPackAction('add-pak')
          },
          {
            key: 'clear-files',
            label: t('pack.removeAll'),
            icon: Trash2,
            action: () => dispatchPackAction('clear-files')
          }
        ]
      },
      {
        key: 'actions',
        label: t('menu.actions'),
        items: [
          {
            key: 'select-export-directory',
            label: t('pack.exportDirectory'),
            icon: FolderOpen,
            action: () => dispatchPackAction('select-export-directory')
          },
          {
            key: 'export',
            label: t('pack.export'),
            icon: Play,
            action: () => dispatchPackAction('export')
          }
        ]
      }
    )
  }

  items.push(
    {
      key: 'tools',
      label: t('menu.tools'),
      items: availableTools.map((tool) => ({
        key: `tool-${tool.id}`,
        label: t(tool.title),
        icon: tool.icon,
        action: () => {
          void router.push(`/tools/${tool.id}`)
        }
      }))
    },
    {
      key: 'settings',
      label: t('menu.settings'),
      items: [
        {
          key: 'open-settings',
          label: t('menu.openSettings'),
          icon: Settings,
          action: () => {
            void router.push('/settings')
          }
        }
      ]
    },
    {
      key: 'help',
      label: t('menu.help'),
      items: [
        {
          key: 'show-update-dialog',
          label: t('updateDialog.updateAvailable'),
          icon: Download,
          action: showUpdateDialog
        },
        {
          key: 'open-github',
          label: 'GitHub',
          icon: Github,
          action: () => {
            void openUrl('https://github.com/eigeen/ree-pak-rs')
          }
        }
      ]
    }
  )

  return items
})

onMounted(async () => {
  try {
    await fileListStore.refreshLocalSource()
  } catch (error) {
    console.error('Failed to refresh file lists:', error)
  }
})
</script>
