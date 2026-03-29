<template>
  <div class="desktop-toolbar">
    <div ref="tabListElement" class="relative flex items-center gap-1">
      <button
        v-for="tab in props.tabs"
        :key="tab.value"
        :ref="(element) => setTabButtonElement(tab.value, element)"
        type="button"
        :class="
          cn(
            'desktop-side-tab relative',
            selectedValue === tab.value && 'desktop-side-tab-active'
          )
        "
        :aria-pressed="selectedValue === tab.value"
        @click="selectedValue = tab.value"
      >
        <component :is="tab.icon" class="size-4" />
        <span>{{ tab.label }}</span>
      </button>

      <span
        aria-hidden="true"
        class="pointer-events-none absolute bottom-0 left-0 h-0.5 rounded-full bg-linear-to-r from-sky-400 via-cyan-300 to-emerald-300 transition-[transform,width,opacity] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]"
        :style="indicatorStyle"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch, type CSSProperties, type Component } from 'vue'
import { cn } from '@/lib/utils'

export interface UnpackSidebarTabItem {
  value: string
  label: string
  icon: Component
}

interface Props {
  tabs: UnpackSidebarTabItem[]
}

const selectedValue = defineModel<string>({ required: true })
const props = defineProps<Props>()
const tabListElement = ref<HTMLElement | null>(null)
const indicatorStyle = ref<CSSProperties>({
  opacity: '0',
  width: '0px',
  transform: 'translate3d(0, 0, 0)'
})
const tabButtonElements = new Map<string, HTMLElement>()
let resizeObserver: ResizeObserver | null = null

function setTabButtonElement(value: string, element: unknown) {
  if (element instanceof HTMLElement) {
    tabButtonElements.set(value, element)
  } else {
    tabButtonElements.delete(value)
  }
}

function updateIndicator() {
  const tabList = tabListElement.value
  const activeButton = tabButtonElements.get(selectedValue.value)

  if (!tabList || !activeButton) {
    indicatorStyle.value = {
      ...indicatorStyle.value,
      opacity: '0',
      width: '0px'
    }
    return
  }

  const inset = 10
  const listRect = tabList.getBoundingClientRect()
  const buttonRect = activeButton.getBoundingClientRect()
  const width = Math.max(buttonRect.width - inset * 2, 0)
  const left = buttonRect.left - listRect.left + inset

  indicatorStyle.value = {
    opacity: '1',
    width: `${width}px`,
    transform: `translate3d(${left}px, 0, 0)`
  }
}

function observeIndicatorTargets() {
  resizeObserver?.disconnect()
  resizeObserver = null

  if (!tabListElement.value) {
    return
  }

  resizeObserver = new ResizeObserver(() => {
    updateIndicator()
  })

  resizeObserver.observe(tabListElement.value)

  for (const element of tabButtonElements.values()) {
    resizeObserver.observe(element)
  }
}

function handleWindowResize() {
  updateIndicator()
}

watch(
  () => [selectedValue.value, props.tabs.map((tab) => tab.value).join('|')] as const,
  async () => {
    await nextTick()
    observeIndicatorTargets()
    updateIndicator()
  },
  { immediate: true }
)

onMounted(() => {
  window.addEventListener('resize', handleWindowResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleWindowResize)
  resizeObserver?.disconnect()
})
</script>
