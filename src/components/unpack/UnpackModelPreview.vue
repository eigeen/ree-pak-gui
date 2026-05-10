<template>
  <div ref="rootRef" :class="modelPreviewRootClass">
    <canvas
      ref="canvasRef"
      class="model-preview-canvas block h-full w-full"
      @contextmenu.prevent
      @pointerdown="handlePointerDown"
      @pointermove="handlePointerMove"
      @pointerup="handlePointerUp"
      @pointercancel="handlePointerUp"
      @wheel.prevent="handleWheel"
    />

    <div
      class="absolute top-3 left-3 grid w-[min(280px,calc(100%_-_24px))] gap-2"
      @pointerdown.stop
      @wheel.stop
    >
      <section :class="previewPanelClass">
        <div :class="previewPanelHeaderClass">
          <button
            type="button"
            :class="previewPanelTitleButtonClass"
            :aria-expanded="settingsPanelExpanded"
            @click="settingsPanelExpanded = !settingsPanelExpanded"
          >
            <ChevronRight
              class="size-3.5 shrink-0 transition-transform duration-150"
              :class="{ 'rotate-90': settingsPanelExpanded }"
            />
            <span :class="previewPanelTitleClass">
              {{ t('unpack.modelPreviewSettings') }}
            </span>
          </button>
        </div>
        <div v-if="settingsPanelExpanded" class="grid gap-2 px-[9px] pb-[9px]">
          <div :class="previewControlRowClass">
            <span :class="previewControlLabelClass">{{
              t('unpack.modelPreviewTextureResolution')
            }}</span>
            <div :class="previewSegmentedClass" role="radiogroup">
              <button
                type="button"
                :class="[
                  previewSegmentClass,
                  meshPreviewTextureResolution === 'standard' && previewSegmentActiveClass
                ]"
                :aria-checked="meshPreviewTextureResolution === 'standard'"
                role="radio"
                @click="meshPreviewTextureResolution = 'standard'"
              >
                {{ t('unpack.modelPreviewTextureResolutionStandard') }}
              </button>
              <button
                type="button"
                :class="[
                  previewSegmentClass,
                  meshPreviewTextureResolution === 'high' && previewSegmentActiveClass
                ]"
                :aria-checked="meshPreviewTextureResolution === 'high'"
                role="radio"
                @click="meshPreviewTextureResolution = 'high'"
              >
                {{ t('unpack.modelPreviewTextureResolutionHigh') }}
              </button>
            </div>
          </div>
          <div :class="previewControlRowClass">
            <span :class="previewControlLabelClass">{{ t('unpack.modelPreviewBackground') }}</span>
            <div :class="previewSegmentedClass" role="radiogroup">
              <button
                type="button"
                :class="[
                  previewSegmentClass,
                  meshPreviewBackgroundStyle === 'dark' && previewSegmentActiveClass
                ]"
                :aria-checked="meshPreviewBackgroundStyle === 'dark'"
                role="radio"
                @click="meshPreviewBackgroundStyle = 'dark'"
              >
                <span :class="[previewSwatchClass, 'bg-[#101215]']" />
                {{ t('unpack.modelPreviewBackgroundDark') }}
              </button>
              <button
                type="button"
                :class="[
                  previewSegmentClass,
                  meshPreviewBackgroundStyle === 'light' && previewSegmentActiveClass
                ]"
                :aria-checked="meshPreviewBackgroundStyle === 'light'"
                role="radio"
                @click="meshPreviewBackgroundStyle = 'light'"
              >
                <span :class="[previewSwatchClass, 'bg-[#dce1e7]']" />
                {{ t('unpack.modelPreviewBackgroundLight') }}
              </button>
            </div>
          </div>
          <label :class="previewControlRowClass">
            <span :class="previewControlLabelClass">{{ t('unpack.modelPreviewGrid') }}</span>
            <Switch v-model="showMeshPreviewGrid" />
          </label>
        </div>
      </section>

      <section v-if="meshItems.length > 0" :class="previewPanelClass">
        <div :class="previewPanelHeaderClass">
          <button
            type="button"
            :class="previewPanelTitleButtonClass"
            :aria-expanded="objectsPanelExpanded"
            @click="objectsPanelExpanded = !objectsPanelExpanded"
          >
            <ChevronRight
              class="size-3.5 shrink-0 transition-transform duration-150"
              :class="{ 'rotate-90': objectsPanelExpanded }"
            />
            <Layers class="size-3.5" />
            <span :class="previewPanelTitleClass">
              {{ t('unpack.modelPreviewObjects') }}
            </span>
          </button>
          <div class="flex items-center gap-1.5">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger as-child>
                  <button
                    type="button"
                    :class="previewActionButtonClass"
                    :aria-label="t('unpack.modelPreviewShowAllMeshes')"
                    @click="showAllMeshes"
                  >
                    <Eye class="size-3.5" />
                  </button>
                </TooltipTrigger>
                <TooltipContent>{{ t('unpack.modelPreviewShowAllMeshes') }}</TooltipContent>
              </Tooltip>
            </TooltipProvider>
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger as-child>
                  <button
                    type="button"
                    :class="previewActionButtonClass"
                    :aria-label="t('unpack.modelPreviewHideAllMeshes')"
                    @click="hideAllMeshes"
                  >
                    <EyeOff class="size-3.5" />
                  </button>
                </TooltipTrigger>
                <TooltipContent>{{ t('unpack.modelPreviewHideAllMeshes') }}</TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </div>
        </div>
        <div
          v-if="objectsPanelExpanded"
          class="max-h-[min(320px,calc(100vh_-_280px))] min-h-0 overflow-auto border-t border-[rgba(255,255,255,0.09)] p-1"
        >
          <div v-for="group in meshGroups" :key="group.key" class="grid gap-0.5">
            <div class="flex min-w-0 items-center gap-1.5 rounded-md">
              <Checkbox
                :model-value="group.visibility"
                class="ml-1.5"
                :aria-label="
                  t('unpack.modelPreviewToggleMeshGroupVisibility', { group: group.name })
                "
                @click.stop
                @update:model-value="toggleMeshGroupVisibility(group, $event === true)"
              >
                <Minus v-if="group.visibility === 'indeterminate'" class="size-3.5" />
                <Check v-else class="size-3.5" />
              </Checkbox>
              <button
                type="button"
                class="flex min-w-0 flex-1 items-center gap-1.5 rounded-md p-1.5 text-left text-xs font-semibold text-[rgba(224,230,238,0.78)] transition-colors duration-150 hover:bg-white/10 hover:text-white"
                :aria-expanded="!collapsedMeshGroupKeys.has(group.key)"
                @click="toggleMeshGroup(group.key)"
              >
                <ChevronRight
                  class="size-3.5 shrink-0 transition-transform duration-150"
                  :class="{
                    'rotate-90': !collapsedMeshGroupKeys.has(group.key)
                  }"
                />
                <span class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">{{
                  group.name
                }}</span>
              </button>
            </div>
            <div
              v-if="!collapsedMeshGroupKeys.has(group.key)"
              class="ml-3 grid gap-0.5 border-l border-white/10 pl-2"
            >
              <label
                v-for="mesh in group.meshes"
                :key="mesh.index"
                class="flex min-w-0 cursor-pointer items-center gap-2 rounded-md p-1.5 text-[rgba(236,241,245,0.9)] transition-[background,opacity] duration-150 hover:bg-[rgba(255,255,255,0.08)]"
                :class="{ 'opacity-[0.55]': !mesh.visible }"
              >
                <Checkbox
                  :model-value="mesh.visible"
                  @update:model-value="toggleMeshVisibility(mesh.index, $event === true)"
                />
                <span class="grid min-w-0 gap-px">
                  <span
                    class="min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-xs text-[rgba(246,248,250,0.94)]"
                    >{{ mesh.name }}</span
                  >
                  <span
                    class="inline-flex min-w-0 items-center gap-1 overflow-hidden text-ellipsis whitespace-nowrap text-[11px] text-[rgba(205,213,224,0.62)]"
                  >
                    <span class="inline-flex items-center gap-[3px]">
                      {{ mesh.vertices }}
                      <Share2 class="size-3" aria-hidden="true" />
                    </span>
                    <span class="text-[rgba(205,213,224,0.42)]">/</span>
                    <span class="inline-flex items-center gap-[3px]">
                      {{ mesh.triangles }}
                      <Triangle class="size-3" aria-hidden="true" />
                    </span>
                  </span>
                </span>
              </label>
            </div>
          </div>
        </div>
      </section>
    </div>

    <div
      class="absolute top-3 right-3 flex gap-1.5 rounded-lg border border-[rgba(255,255,255,0.12)] bg-[rgba(14,16,20,0.72)] p-1 backdrop-blur-[10px]"
    >
      <button
        type="button"
        class="inline-flex size-[30px] items-center justify-center rounded-md text-[rgba(236,241,245,0.86)] transition-colors duration-150 hover:bg-white/12 hover:text-white"
        :title="t('unpack.modelPreviewResetView')"
        @click="resetCamera"
      >
        <RotateCcw class="size-4" />
      </button>
    </div>

    <div
      v-if="statusText"
      class="absolute bottom-3 left-3 flex max-w-[min(520px,calc(100%_-_24px))] items-center gap-2 rounded-lg border border-[rgba(255,255,255,0.12)] bg-[rgba(14,16,20,0.72)] px-2.5 py-2 text-xs leading-[1.35] text-[rgba(236,241,245,0.9)] backdrop-blur-[10px]"
    >
      <Loader2 v-if="loading" class="size-4 animate-spin" />
      <AlertTriangle v-else-if="error" class="size-4" />
      <Box v-else class="size-4" />
      <span class="min-w-0 [overflow-wrap:anywhere]">{{ statusText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, shallowRef, unref, watch, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  AlertTriangle,
  Box,
  Check,
  ChevronRight,
  Eye,
  EyeOff,
  Layers,
  Loader2,
  Minus,
  RotateCcw,
  Share2,
  Triangle
} from 'lucide-vue-next'
import { Checkbox } from '@/components/ui/checkbox'
import { Switch } from '@/components/ui/switch'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import type { ExplorerEntry } from '@/lib/unpackExplorer'
import {
  loadModelPreviewGeometry,
  loadModelPreviewTextureImages,
  type ModelPreviewGeometry
} from '@/lib/modelInsight/loader'
import { isModelInsightWasmUnavailableError, type PreviewModel } from '@/lib/modelInsight/wasm'
import { type ModelTextureImages } from '@/lib/modelInsight/textures'
import { logFrontendWarn } from '@/utils/frontendLog'
import { ShowWarn } from '@/utils/message'
import {
  addVec3,
  createDefaultModelPreviewCamera,
  createModelPreviewRenderState,
  disposeModelPreviewRenderState,
  modelPreviewCameraVectors,
  renderModelPreviewFrame,
  scaleVec3,
  setModelPreviewMeshVisibility,
  setModelPreviewRenderMeshes,
  type ModelPreviewCamera,
  type ModelPreviewRenderState,
  type Vec3
} from '@/lib/modelInsight/webglRenderer'
import {
  useSettingsStore,
  type AppSettings,
  type MeshPreviewBackgroundStyle,
  type MeshPreviewTextureResolution
} from '@/store/settings'

const props = defineProps<{
  entry: ExplorerEntry
}>()

const { t } = useI18n()
const settingsStore = useSettingsStore()
const settings = computed(() => unref(settingsStore.settings as unknown as Ref<AppSettings>))
const rootRef = ref<HTMLDivElement | null>(null)
const canvasRef = ref<HTMLCanvasElement | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const preview = shallowRef<PreviewModel | null>(null)
const visibleMeshIndexes = ref<Set<number>>(new Set())
const settingsPanelExpanded = ref(false)
const objectsPanelExpanded = ref(true)
const collapsedMeshGroupKeys = ref<Set<string>>(new Set())

type MeshPreviewItem = {
  index: number
  groupKey: string
  groupName: string
  name: string
  vertices: number
  triangles: number
  visible: boolean
}

type MeshPreviewVisibility = boolean | 'indeterminate'

type MeshPreviewGroup = {
  key: string
  name: string
  meshes: MeshPreviewItem[]
  visibility: MeshPreviewVisibility
}

let glState: ModelPreviewRenderState | null = null
let resizeObserver: ResizeObserver | null = null
let animationFrame = 0
let previewLoadToken = 0
let pointerState: {
  id: number
  x: number
  y: number
  button: number
} | null = null

const previewPanelClass =
  'rounded-lg border border-[rgba(255,255,255,0.12)] bg-[rgba(14,16,20,0.74)] text-[12px] leading-[1.35] text-[rgba(236,241,245,0.9)] shadow-[0_16px_42px_rgba(0,0,0,0.22)] backdrop-blur-[10px]'
const previewPanelHeaderClass = 'flex items-center justify-between gap-2.5 p-[9px]'
const previewPanelTitleButtonClass =
  'flex min-w-0 items-center gap-1.5 rounded-md text-[rgba(246,248,250,0.94)] transition-colors duration-150 hover:text-white'
const previewPanelTitleClass =
  'min-w-0 overflow-hidden text-ellipsis whitespace-nowrap text-[11px] font-semibold text-[rgba(246,248,250,0.94)]'
const previewControlRowClass = 'flex items-center justify-between gap-2.5'
const previewControlLabelClass = 'min-w-max text-[rgba(205,213,224,0.76)]'
const previewSegmentedClass =
  'inline-flex min-w-0 overflow-hidden rounded-md border border-[rgba(255,255,255,0.1)] bg-[rgba(255,255,255,0.055)]'
const previewSegmentClass =
  'inline-flex min-w-0 items-center gap-[5px] px-[7px] py-1 text-[11px] text-[rgba(224,230,238,0.78)] transition-colors duration-150 hover:bg-white/10 hover:text-white'
const previewSegmentActiveClass = 'bg-[rgba(255,255,255,0.15)] text-white'
const previewSwatchClass = 'size-3 shrink-0 rounded-full border border-[rgba(255,255,255,0.26)]'
const previewActionButtonClass =
  'flex size-[26px] items-center justify-center rounded-md text-[rgba(224,230,238,0.78)] transition-colors duration-150 hover:bg-white/10 hover:text-white'

const camera: ModelPreviewCamera = {
  target: [0, 0, 0] as Vec3,
  yaw: -0.65,
  pitch: -0.48,
  distance: 1
}

const meshPreviewBackgroundStyle = computed<MeshPreviewBackgroundStyle>({
  get: () => settings.value?.preview?.meshPreview?.backgroundStyle ?? 'dark',
  set: (value) => {
    if (!settings.value?.preview?.meshPreview) return
    settings.value.preview.meshPreview.backgroundStyle = value
  }
})

const showMeshPreviewGrid = computed({
  get: () => settings.value?.preview?.meshPreview?.showGrid ?? true,
  set: (value: boolean) => {
    if (!settings.value?.preview?.meshPreview) return
    settings.value.preview.meshPreview.showGrid = value
  }
})

const meshPreviewTextureResolution = computed<MeshPreviewTextureResolution>({
  get: () => settings.value?.preview?.meshPreview?.textureResolution ?? 'standard',
  set: (value) => {
    if (!settings.value?.preview?.meshPreview) return
    settings.value.preview.meshPreview.textureResolution = value
  }
})

const modelPreviewRootClass = computed(() => [
  'model-preview relative h-full min-h-0 w-full overflow-hidden',
  `model-preview-background-${meshPreviewBackgroundStyle.value}`,
  { 'model-preview-grid': showMeshPreviewGrid.value }
])

const meshItems = computed(() => {
  const model = preview.value
  if (!model) return []

  return model.meshes.map((mesh, index) => {
    const trimmedName = mesh.name.trim()
    const name = trimmedName || t('unpack.modelPreviewUnnamedSubmesh', { index: index + 1 })
    const group = resolveMeshGroup(name)
    return {
      index,
      groupKey: group.key,
      groupName: group.name,
      name: group.itemName,
      vertices: mesh.positions.length,
      triangles: Math.floor(mesh.indices.length / 3),
      visible: visibleMeshIndexes.value.has(index)
    }
  })
})

const meshGroups = computed<MeshPreviewGroup[]>(() => {
  const groups = new Map<string, MeshPreviewGroup>()
  for (const mesh of meshItems.value) {
    const existing = groups.get(mesh.groupKey)
    if (existing) {
      existing.meshes.push(mesh)
      continue
    }
    groups.set(mesh.groupKey, {
      key: mesh.groupKey,
      name: mesh.groupName,
      meshes: [mesh],
      visibility: false
    })
  }
  return Array.from(groups.values()).map((group) => ({
    ...group,
    visibility: resolveMeshGroupVisibility(group.meshes)
  }))
})

const statusText = computed(() => {
  if (loading.value) return t('unpack.modelPreviewLoading')
  if (error.value) return error.value
  const model = preview.value
  if (!model) return ''
  const vertices = model.meshes.reduce((sum, mesh) => sum + mesh.positions.length, 0)
  const triangles = model.meshes.reduce((sum, mesh) => sum + Math.floor(mesh.indices.length / 3), 0)
  return t('unpack.modelPreviewStats', {
    submeshes: model.meshes.length,
    vertices,
    triangles
  })
})

onMounted(() => {
  setupCanvas()
  void loadPreview()
})

onUnmounted(() => {
  if (animationFrame) window.cancelAnimationFrame(animationFrame)
  resizeObserver?.disconnect()
  disposeRenderState()
})

watch(
  () => props.entry.id,
  () => {
    void loadPreview()
  }
)

watch(meshPreviewTextureResolution, () => {
  if (!preview.value || loading.value) return
  void loadPreview()
})

async function loadPreview() {
  const entry = props.entry
  const loadToken = ++previewLoadToken
  if (!entry.hash || !entry.belongsTo) {
    error.value = t('unpack.modelInsightMissingSource')
    return
  }

  loading.value = true
  error.value = null

  try {
    const geometry = await loadModelPreviewGeometry(entry)
    if (loadToken !== previewLoadToken) return
    preview.value = geometry.preview
    resetVisibleMeshes(geometry.preview)
    buildRenderState(geometry.preview)
    resetCamera()
    loading.value = false
    void loadPreviewTextures(loadToken, geometry)
  } catch (caught) {
    if (loadToken !== previewLoadToken) return
    const message = modelPreviewErrorMessage(caught)
    error.value = message
  } finally {
    if (loadToken === previewLoadToken) {
      loading.value = false
    }
  }
}

async function loadPreviewTextures(loadToken: number, geometry: ModelPreviewGeometry) {
  const textureImages = await loadPreviewTextureImages(geometry)
  if (loadToken !== previewLoadToken || preview.value !== geometry.preview) return
  if (Object.keys(textureImages).length === 0) return
  buildRenderState(geometry.preview, textureImages)
}

async function loadPreviewTextureImages(geometry: ModelPreviewGeometry) {
  try {
    return await loadModelPreviewTextureImages(props.entry, geometry, {
      textureResolution: meshPreviewTextureResolution.value,
      warnScope: 'unpack.modelPreview'
    })
  } catch (caught) {
    logFrontendWarn(
      'unpack.modelPreview',
      `texture load failed path=${geometry.assets.mdfEntryPath ?? geometry.assets.meshEntryPath} error=${caught instanceof Error ? caught.message : String(caught)}`
    )
    return {}
  }
}

function modelPreviewErrorMessage(caught: unknown) {
  if (isModelInsightWasmUnavailableError(caught)) {
    const message = t('unpack.modelPreviewUnavailable')
    ShowWarn(message)
    return message
  }

  return caught instanceof Error ? caught.message : String(caught)
}

function setupCanvas() {
  const root = rootRef.value
  const canvas = canvasRef.value
  if (!root || !canvas) return

  const gl = canvas.getContext('webgl', {
    antialias: true,
    alpha: true
  })
  if (!gl) {
    error.value = t('unpack.modelPreviewWebglUnavailable')
    return
  }

  try {
    glState = createModelPreviewRenderState(gl, {
      indexUnavailable: t('unpack.modelPreviewIndexUnavailable'),
      bufferFailed: t('unpack.modelPreviewBufferFailed'),
      programFailed: t('unpack.modelPreviewProgramFailed'),
      shaderFailed: t('unpack.modelPreviewShaderFailed')
    })
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught)
    return
  }
  resizeObserver = new ResizeObserver(() => resizeCanvas())
  resizeObserver.observe(root)
  resizeCanvas()
  renderLoop()
}

function buildRenderState(model: PreviewModel, textureImages: ModelTextureImages = {}) {
  if (!glState) return
  setModelPreviewRenderMeshes(glState, model, textureImages)
  applyMeshVisibility()
}

function resetCamera() {
  const model = preview.value
  if (!model) return
  const nextCamera = createDefaultModelPreviewCamera(model)
  camera.target = nextCamera.target
  camera.yaw = nextCamera.yaw
  camera.pitch = nextCamera.pitch
  camera.distance = nextCamera.distance
}

function resetVisibleMeshes(model: PreviewModel) {
  visibleMeshIndexes.value = new Set(model.meshes.map((_, index) => index))
  collapsedMeshGroupKeys.value = new Set()
}

function applyMeshVisibility() {
  if (!glState) return
  setModelPreviewMeshVisibility(glState, visibleMeshIndexes.value)
}

function toggleMeshVisibility(index: number, visible: boolean) {
  const nextVisibleIndexes = new Set(visibleMeshIndexes.value)
  if (visible) {
    nextVisibleIndexes.add(index)
  } else {
    nextVisibleIndexes.delete(index)
  }
  visibleMeshIndexes.value = nextVisibleIndexes
  applyMeshVisibility()
}

function toggleMeshGroupVisibility(group: MeshPreviewGroup, visible: boolean) {
  const nextVisibleIndexes = new Set(visibleMeshIndexes.value)
  for (const mesh of group.meshes) {
    if (visible) {
      nextVisibleIndexes.add(mesh.index)
    } else {
      nextVisibleIndexes.delete(mesh.index)
    }
  }
  visibleMeshIndexes.value = nextVisibleIndexes
  applyMeshVisibility()
}

function showAllMeshes() {
  const model = preview.value
  if (!model) return
  resetVisibleMeshes(model)
  applyMeshVisibility()
}

function hideAllMeshes() {
  visibleMeshIndexes.value = new Set()
  applyMeshVisibility()
}

function toggleMeshGroup(key: string) {
  const nextCollapsedKeys = new Set(collapsedMeshGroupKeys.value)
  if (nextCollapsedKeys.has(key)) {
    nextCollapsedKeys.delete(key)
  } else {
    nextCollapsedKeys.add(key)
  }
  collapsedMeshGroupKeys.value = nextCollapsedKeys
}

function resolveMeshGroupVisibility(meshes: MeshPreviewItem[]): MeshPreviewVisibility {
  const visibleCount = meshes.filter((mesh) => mesh.visible).length
  if (visibleCount === 0) return false
  if (visibleCount === meshes.length) return true
  return 'indeterminate'
}

function resolveMeshGroup(name: string) {
  const submeshMatch = name.match(/^(.+?)[_. -]+(sub(?:mesh)?\d+(?:[_. -].*)?)$/i)
  if (submeshMatch?.[1] && submeshMatch[2]) {
    const groupName = trimMeshGroupName(submeshMatch[1])
    return {
      key: `name:${groupName}`,
      name: groupName,
      itemName: trimMeshItemName(submeshMatch[2])
    }
  }

  const doubleSeparatorIndex = name.indexOf('__')
  if (doubleSeparatorIndex > 0) {
    const groupName = trimMeshGroupName(name.slice(0, doubleSeparatorIndex))
    return {
      key: `name:${groupName}`,
      name: groupName,
      itemName: trimMeshItemName(name.slice(doubleSeparatorIndex + 2))
    }
  }

  return {
    key: 'ungrouped',
    name: t('unpack.modelPreviewUngroupedMeshes'),
    itemName: name
  }
}

function trimMeshGroupName(value: string) {
  return value.replace(/[_. -]+$/g, '') || value
}

function trimMeshItemName(value: string) {
  const trimmed = value.replace(/^[_. -]+/g, '').trim()
  return trimmed || value
}

function handlePointerDown(event: PointerEvent) {
  const canvas = canvasRef.value
  if (!canvas) return
  pointerState = {
    id: event.pointerId,
    x: event.clientX,
    y: event.clientY,
    button: event.button
  }
  canvas.setPointerCapture(event.pointerId)
}

function handlePointerMove(event: PointerEvent) {
  if (!pointerState || pointerState.id !== event.pointerId) return
  const dx = event.clientX - pointerState.x
  const dy = event.clientY - pointerState.y
  pointerState.x = event.clientX
  pointerState.y = event.clientY

  if (pointerState.button === 2) {
    panCamera(dx, dy)
    return
  }

  camera.yaw -= dx * 0.006
  camera.pitch = clamp(camera.pitch + dy * 0.005, -1.45, 1.45)
}

function handlePointerUp(event: PointerEvent) {
  if (!pointerState || pointerState.id !== event.pointerId) return
  canvasRef.value?.releasePointerCapture(event.pointerId)
  pointerState = null
}

function handleWheel(event: WheelEvent) {
  const scroll = event.deltaMode === WheelEvent.DOM_DELTA_LINE ? event.deltaY : event.deltaY / 16
  camera.distance = clamp(camera.distance * (1 + scroll * 0.035), 0.05, 100000)
}

function panCamera(dx: number, dy: number) {
  const view = modelPreviewCameraVectors(camera)
  const scale = camera.distance * 0.0015
  camera.target = addVec3(camera.target, scaleVec3(view.right, -dx * scale))
  camera.target = addVec3(camera.target, scaleVec3(view.up, dy * scale))
}

function resizeCanvas() {
  const root = rootRef.value
  const canvas = canvasRef.value
  const state = glState
  if (!root || !canvas || !state) return

  const rect = root.getBoundingClientRect()
  const ratio = window.devicePixelRatio || 1
  const width = Math.max(1, Math.floor(rect.width * ratio))
  const height = Math.max(1, Math.floor(rect.height * ratio))
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width
    canvas.height = height
  }
  state.gl.viewport(0, 0, width, height)
}

function renderLoop() {
  render()
  animationFrame = window.requestAnimationFrame(renderLoop)
}

function render() {
  const state = glState
  if (!state) return
  resizeCanvas()
  renderModelPreviewFrame(state, camera)
}

function disposeRenderState() {
  const state = glState
  if (!state) return
  disposeModelPreviewRenderState(state)
  glState = null
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max)
}
</script>

<style scoped>
.model-preview-background-dark {
  background: #101215;
}

.model-preview-background-light {
  background: #dce1e7;
}

.model-preview-grid {
  background:
    linear-gradient(90deg, rgba(255, 255, 255, 0.035) 1px, transparent 1px),
    linear-gradient(0deg, rgba(255, 255, 255, 0.035) 1px, transparent 1px), #101215;
  background-size: 28px 28px;
}

.model-preview-background-light.model-preview-grid {
  background:
    linear-gradient(90deg, rgba(49, 55, 66, 0.105) 1px, transparent 1px),
    linear-gradient(0deg, rgba(49, 55, 66, 0.105) 1px, transparent 1px), #dce1e7;
}

.model-preview-canvas {
  touch-action: none;
  cursor: grab;
}

.model-preview-canvas:active {
  cursor: grabbing;
}
</style>
