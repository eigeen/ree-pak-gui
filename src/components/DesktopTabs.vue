<template>
  <div ref="tabListElement" :class="cn('relative flex items-center gap-1', props.class)">
    <component
      :is="tab.to ? RouterLink : 'button'"
      v-for="tab in props.tabs"
      :key="tab.value"
      :ref="
        (element: Element | ComponentPublicInstance | null) =>
          setTabButtonElement(tab.value, element)
      "
      v-bind="tab.to ? { to: tab.to } : { type: 'button' }"
      :class="
        cn('desktop-side-tab relative', selectedValue === tab.value && 'desktop-side-tab-active')
      "
      :aria-pressed="!tab.to ? selectedValue === tab.value : undefined"
      :aria-current="tab.to && selectedValue === tab.value ? 'page' : undefined"
      @click="handleSelect(tab.value)"
    >
      <component :is="tab.icon" v-if="tab.icon" class="size-4" />
      <span>{{ tab.label }}</span>
    </component>

    <span
      aria-hidden="true"
      class="pointer-events-none absolute bottom-0 left-0 h-0.5 rounded-full bg-linear-to-r from-sky-400 via-cyan-300 to-emerald-300 transition-[transform,width,opacity] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)]"
      :style="indicatorStyle"
    />
  </div>
</template>

<script setup lang="ts">
import {
  nextTick,
  onMounted,
  onUnmounted,
  ref,
  watch,
  type CSSProperties,
  type Component,
  type ComponentPublicInstance
} from 'vue'
import { RouterLink, type RouteLocationRaw } from 'vue-router'
import { cn } from '@/lib/utils'

export interface DesktopTabItem {
  value: string
  label: string
  icon?: Component
  to?: RouteLocationRaw
}

interface Props {
  class?: string
  tabs: DesktopTabItem[]
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
  const resolvedElement =
    element instanceof HTMLElement
      ? element
      : element &&
          typeof element === 'object' &&
          '$el' in element &&
          element.$el instanceof HTMLElement
        ? element.$el
        : null

  if (resolvedElement) {
    tabButtonElements.set(value, resolvedElement)
  } else {
    tabButtonElements.delete(value)
  }
}

function handleSelect(value: string) {
  selectedValue.value = value
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
  void nextTick().then(() => {
    observeIndicatorTargets()
    updateIndicator()
  })
})

onUnmounted(() => {
  window.removeEventListener('resize', handleWindowResize)
  resizeObserver?.disconnect()
})
</script>
