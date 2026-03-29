<template>
  <div ref="rootRef" class="relative flex min-w-0 items-center gap-0">
    <button
      v-for="item in props.items"
      :key="item.key"
      :ref="(el) => setTriggerRef(item.key, el)"
      type="button"
      class="desktop-menu-trigger"
      :class="openKey === item.key ? 'bg-secondary/80 text-foreground' : ''"
      @click="toggleMenu(item.key)"
      @pointerenter="handleTriggerPointerEnter(item.key)"
    >
      {{ item.label }}
    </button>

    <div
      v-if="activeMenu"
      class="absolute top-full z-50 pt-1"
      :style="activeMenuStyle"
    >
      <div
        class="min-w-[14rem] overflow-hidden rounded-md border border-border/80 bg-popover p-1 text-popover-foreground shadow-md"
        @pointerleave="handleMenuPointerLeave"
      >
        <button
          v-for="entry in activeMenu.items"
          :key="entry.key"
          type="button"
          class="flex w-full items-center gap-2 rounded-sm px-3 py-2 text-left text-sm transition-colors hover:bg-accent hover:text-accent-foreground"
          :class="entry.destructive ? 'text-destructive hover:bg-destructive/10 hover:text-destructive' : ''"
          @click="selectItem(entry)"
        >
          <component :is="entry.icon" v-if="entry.icon" class="size-4 shrink-0" />
          <span class="truncate">{{ entry.label }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'

type MenuEntry = {
  key: string
  label: string
  icon?: unknown
  destructive?: boolean
  action: () => void | Promise<void>
}

type MenuGroup = {
  key: string
  label: string
  items: MenuEntry[]
}

const props = defineProps<{
  items: MenuGroup[]
}>()

const rootRef = ref<HTMLElement | null>(null)
const triggerRefs = ref<Record<string, HTMLElement | null>>({})
const openKey = ref('')

const activeMenu = computed(() => props.items.find((item) => item.key === openKey.value) ?? null)
const activeMenuStyle = computed(() => {
  const trigger = openKey.value ? triggerRefs.value[openKey.value] : null
  if (!trigger || !rootRef.value) {
    return {}
  }

  const rootRect = rootRef.value.getBoundingClientRect()
  const triggerRect = trigger.getBoundingClientRect()

  return {
    left: `${triggerRect.left - rootRect.left}px`
  }
})

function setTriggerRef(key: string, element: Element | null) {
  triggerRefs.value[key] = element instanceof HTMLElement ? element : null
}

function toggleMenu(key: string) {
  openKey.value = openKey.value === key ? '' : key
}

function handleTriggerPointerEnter(key: string) {
  if (!openKey.value || openKey.value === key) {
    return
  }

  openKey.value = key
}

function handleMenuPointerLeave() {
  // VS Code 风格不会在离开菜单时立刻关闭，留给外部点击/再次点击处理。
}

function selectItem(entry: MenuEntry) {
  openKey.value = ''
  entry.action()
}

function handleWindowPointerDown(event: PointerEvent) {
  if (!openKey.value) {
    return
  }

  const target = event.target
  if (target instanceof Node && rootRef.value?.contains(target)) {
    return
  }

  openKey.value = ''
}

function handleWindowKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    openKey.value = ''
  }
}

onMounted(() => {
  window.addEventListener('pointerdown', handleWindowPointerDown, true)
  window.addEventListener('keydown', handleWindowKeydown)
})

onUnmounted(() => {
  window.removeEventListener('pointerdown', handleWindowPointerDown, true)
  window.removeEventListener('keydown', handleWindowKeydown)
})
</script>
