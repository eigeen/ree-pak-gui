<template>
  <header class="sticky top-0 z-40 px-4 pt-4 sm:px-6 lg:px-8">
    <div
      class="app-container rounded-[1.5rem] border border-white/60 bg-background/72 px-4 py-3 backdrop-blur-xl transition-all duration-300 sm:px-5"
      :class="scrollY > 12 ? 'glass-ring shadow-[0_20px_50px_-36px_rgba(15,23,42,0.7)]' : ''"
    >
      <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div class="flex min-w-0 items-center gap-4">
          <div
            class="flex size-11 shrink-0 items-center justify-center rounded-2xl border border-primary/20 bg-primary/10 text-primary"
          >
            <PackageOpen class="size-5" />
          </div>
          <div class="min-w-0">
            <p class="section-eyebrow">REE Pak GUI</p>
            <div class="flex items-center gap-2">
              <h1
                class="truncate text-base font-semibold tracking-tight text-foreground sm:text-lg"
              >
                {{ t('menu.slogan') }}
              </h1>
              <Badge
                variant="outline"
                class="hidden border-accent/60 bg-accent/40 text-accent-foreground md:inline-flex"
              >
                shadcn-vue
              </Badge>
            </div>
          </div>
        </div>

        <div class="flex flex-col gap-3 lg:flex-row lg:items-center">
          <nav class="flex flex-wrap items-center gap-2">
            <RouterLink :class="navLinkClass('/unpack')" :to="{ name: 'UnpackView' }">
              {{ t('menu.unpack') }}
            </RouterLink>
            <RouterLink :class="navLinkClass('/pack')" :to="{ name: 'PackView' }">
              {{ t('menu.repack') }}
            </RouterLink>

            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <Button variant="ghost" size="sm" class="rounded-full px-4">
                  {{ t('menu.tools') }}
                  <ChevronDown class="size-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" class="w-60 rounded-2xl">
                <DropdownMenuItem
                  v-for="tool in availableTools"
                  :key="tool.id"
                  class="cursor-pointer rounded-xl px-3 py-2"
                  @select="router.push(`/tools/${tool.id}`)"
                >
                  <component :is="tool.icon" v-if="tool.icon" class="size-4" />
                  <span>{{ t(tool.title) }}</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </nav>

          <div class="flex items-center gap-2">
            <Button
              v-if="updateStore.updateVersion"
              variant="outline"
              size="sm"
              class="relative rounded-full"
              @click="showUpdateDialog"
            >
              <Download class="size-4" />
              <span>{{ t('updateDialog.updateAvailable') }}</span>
              <span class="absolute right-2.5 top-2.5 size-2 rounded-full bg-destructive" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="rounded-full"
              @click="openUrl('https://github.com/eigeen/ree-pak-rs')"
            >
              <Github class="size-4" />
            </Button>
          </div>
        </div>
      </div>
    </div>

    <UpdateDialog ref="updateDialog" />
  </header>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { openUrl } from '@tauri-apps/plugin-opener'
import { ChevronDown, Download, Github, PackageOpen } from 'lucide-vue-next'
import { cn } from '@/lib/utils'
import { getAllTools } from '@/config/tools'
import { useUpdateStore } from '@/store/update'
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
const updateStore = useUpdateStore()

const updateDialog = ref<{ popup: () => void } | null>(null)
const scrollY = ref(0)
const availableTools = getAllTools()

const navLinkClass = (path: string) =>
  cn(
    buttonVariants({ variant: route.path === path ? 'default' : 'ghost', size: 'sm' }),
    'rounded-full px-4'
  )

const showUpdateDialog = () => {
  updateDialog.value?.popup()
}

const handleScroll = () => {
  scrollY.value = window.scrollY
}

onMounted(() => {
  window.addEventListener('scroll', handleScroll, { passive: true })
  handleScroll()
})

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll)
})
</script>
