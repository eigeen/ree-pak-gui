<template>
  <header class="desktop-header">
    <div class="desktop-menubar">
      <div class="flex min-w-0 items-center gap-5">
        <div class="desktop-brand">
          <PackageOpen class="size-4" />
          <span>REE PAK</span>
          <span class="desktop-brand-version">Workbench</span>
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
          <Download class="size-3.5" />
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
      <div class="flex min-w-0 items-center gap-1">
        <template v-if="isUnpackView">
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <button class="desktop-menu-trigger" type="button">
                {{ t('menu.resources') }}
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" class="w-56 rounded-md border-border/80">
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="openPathListManager"
              >
                <Wrench class="size-4" />
                <span>{{ t('menu.managePathLists') }}</span>
              </DropdownMenuItem>
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="dispatchUnpackAction('open-paks')"
              >
                <FolderOpen class="size-4" />
                <span>{{ t('menu.openPaks') }}</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <button class="desktop-menu-trigger" type="button">
                {{ t('menu.actions') }}
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" class="w-56 rounded-md border-border/80">
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="dispatchUnpackAction('render-tree')"
              >
                <RefreshCw class="size-4" />
                <span>{{ t('menu.reloadTree') }}</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </template>

        <template v-else-if="isPackView">
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <button class="desktop-menu-trigger" type="button">
                {{ t('menu.resources') }}
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" class="w-56 rounded-md border-border/80">
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="dispatchPackAction('add-folder')"
              >
                <FolderPlus class="size-4" />
                <span>{{ t('pack.addFolder') }}</span>
              </DropdownMenuItem>
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="dispatchPackAction('add-pak')"
              >
                <PackagePlus class="size-4" />
                <span>{{ t('pack.addPak') }}</span>
              </DropdownMenuItem>
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="dispatchPackAction('clear-files')"
              >
                <Trash2 class="size-4" />
                <span>{{ t('pack.removeAll') }}</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <button class="desktop-menu-trigger" type="button">
                {{ t('menu.actions') }}
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" class="w-56 rounded-md border-border/80">
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="dispatchPackAction('select-export-directory')"
              >
                <FolderOpen class="size-4" />
                <span>{{ t('pack.exportDirectory') }}</span>
              </DropdownMenuItem>
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="dispatchPackAction('export')"
              >
                <Play class="size-4" />
                <span>{{ t('pack.export') }}</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </template>

        <template v-if="!isSettingsView">
          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <button class="desktop-menu-trigger" type="button">
                {{ t('menu.tools') }}
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" class="w-60 rounded-md border-border/80">
              <DropdownMenuItem
                v-for="tool in availableTools"
                :key="tool.id"
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="router.push(`/tools/${tool.id}`)"
              >
                <component :is="tool.icon" v-if="tool.icon" class="size-4" />
                <span>{{ t(tool.title) }}</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <button class="desktop-menu-trigger" type="button">
                {{ t('menu.settings') }}
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" class="w-56 rounded-md border-border/80">
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="router.push('/settings')"
              >
                <Settings class="size-4" />
                <span>{{ t('menu.openSettings') }}</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          <DropdownMenu>
            <DropdownMenuTrigger as-child>
              <button class="desktop-menu-trigger" type="button">
                {{ t('menu.help') }}
              </button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" class="w-56 rounded-md border-border/80">
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="showUpdateDialog"
              >
                <Download class="size-4" />
                <span>{{ t('updateDialog.updateAvailable') }}</span>
              </DropdownMenuItem>
              <DropdownMenuItem
                class="cursor-pointer rounded-sm px-3 py-2"
                @select="openUrl('https://github.com/eigeen/ree-pak-rs')"
              >
                <Github class="size-4" />
                <span>GitHub</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </template>
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
import FileNameTable from '@/components/FileNameTable/FileNameTable.vue'
import { useFileListStore } from '@/store/filelist'
import { useUpdateStore } from '@/store/update'
import { useWorkStore } from '@/store/work'
import { Badge } from '@/components/ui/badge'
import { Button, buttonVariants } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'

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

onMounted(async () => {
  try {
    await fileListStore.refreshLocalSource()
  } catch (error) {
    console.error('Failed to refresh file lists:', error)
  }
})
</script>
