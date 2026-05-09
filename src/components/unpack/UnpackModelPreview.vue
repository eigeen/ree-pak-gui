<template>
  <div ref="rootRef" :class="modelPreviewRootClass">
    <canvas
      ref="canvasRef"
      class="model-preview-canvas"
      @contextmenu.prevent
      @pointerdown="handlePointerDown"
      @pointermove="handlePointerMove"
      @pointerup="handlePointerUp"
      @pointercancel="handlePointerUp"
      @wheel.prevent="handleWheel"
    />

    <div class="model-preview-left-ui" @pointerdown.stop @wheel.stop>
      <section class="model-preview-panel model-preview-settings-panel">
        <div class="model-preview-panel-header">
          <button
            type="button"
            class="model-preview-panel-title-button"
            :aria-expanded="settingsPanelExpanded"
            @click="settingsPanelExpanded = !settingsPanelExpanded"
          >
            <ChevronRight
              class="model-preview-chevron size-3.5"
              :class="{ 'model-preview-chevron-expanded': settingsPanelExpanded }"
            />
            <span class="model-preview-panel-title">
              {{ t('unpack.modelPreviewSettings') }}
            </span>
          </button>
        </div>
        <div v-if="settingsPanelExpanded" class="model-preview-settings-body">
          <div class="model-preview-control-row">
            <span>{{ t('unpack.modelPreviewTextureResolution') }}</span>
            <div class="model-preview-segmented" role="radiogroup">
              <button
                type="button"
                class="model-preview-segment"
                :class="{
                  'model-preview-segment-active': meshPreviewTextureResolution === 'standard'
                }"
                :aria-checked="meshPreviewTextureResolution === 'standard'"
                role="radio"
                @click="meshPreviewTextureResolution = 'standard'"
              >
                {{ t('unpack.modelPreviewTextureResolutionStandard') }}
              </button>
              <button
                type="button"
                class="model-preview-segment"
                :class="{ 'model-preview-segment-active': meshPreviewTextureResolution === 'high' }"
                :aria-checked="meshPreviewTextureResolution === 'high'"
                role="radio"
                @click="meshPreviewTextureResolution = 'high'"
              >
                {{ t('unpack.modelPreviewTextureResolutionHigh') }}
              </button>
            </div>
          </div>
          <div class="model-preview-control-row">
            <span>{{ t('unpack.modelPreviewBackground') }}</span>
            <div class="model-preview-segmented" role="radiogroup">
              <button
                type="button"
                class="model-preview-segment"
                :class="{ 'model-preview-segment-active': meshPreviewBackgroundStyle === 'dark' }"
                :aria-checked="meshPreviewBackgroundStyle === 'dark'"
                role="radio"
                @click="meshPreviewBackgroundStyle = 'dark'"
              >
                <span class="model-preview-swatch model-preview-swatch-dark" />
                {{ t('unpack.modelPreviewBackgroundDark') }}
              </button>
              <button
                type="button"
                class="model-preview-segment"
                :class="{ 'model-preview-segment-active': meshPreviewBackgroundStyle === 'light' }"
                :aria-checked="meshPreviewBackgroundStyle === 'light'"
                role="radio"
                @click="meshPreviewBackgroundStyle = 'light'"
              >
                <span class="model-preview-swatch model-preview-swatch-light" />
                {{ t('unpack.modelPreviewBackgroundLight') }}
              </button>
            </div>
          </div>
          <label class="model-preview-control-row model-preview-switch-row">
            <span>{{ t('unpack.modelPreviewGrid') }}</span>
            <Switch v-model="showMeshPreviewGrid" />
          </label>
        </div>
      </section>

      <section v-if="meshItems.length > 0" class="model-preview-panel model-preview-objects-panel">
        <div class="model-preview-panel-header">
          <button
            type="button"
            class="model-preview-panel-title-button"
            :aria-expanded="objectsPanelExpanded"
            @click="objectsPanelExpanded = !objectsPanelExpanded"
          >
            <ChevronRight
              class="model-preview-chevron size-3.5"
              :class="{ 'model-preview-chevron-expanded': objectsPanelExpanded }"
            />
            <Layers class="size-3.5" />
            <span class="model-preview-panel-title">
              {{ t('unpack.modelPreviewObjects') }}
            </span>
          </button>
          <div class="model-preview-object-actions">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger as-child>
                  <button
                    type="button"
                    class="model-preview-action-button"
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
                    class="model-preview-action-button"
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
        <div v-if="objectsPanelExpanded" class="model-preview-object-list">
          <div v-for="group in meshGroups" :key="group.key" class="model-preview-object-group">
            <button
              type="button"
              class="model-preview-group-row"
              :aria-expanded="!collapsedMeshGroupKeys.has(group.key)"
              @click="toggleMeshGroup(group.key)"
            >
              <ChevronRight
                class="model-preview-chevron size-3.5"
                :class="{
                  'model-preview-chevron-expanded': !collapsedMeshGroupKeys.has(group.key)
                }"
              />
              <span class="model-preview-group-name">{{ group.name }}</span>
            </button>
            <div v-if="!collapsedMeshGroupKeys.has(group.key)" class="model-preview-group-children">
              <label
                v-for="mesh in group.meshes"
                :key="mesh.index"
                class="model-preview-object-row"
                :class="{ 'model-preview-object-row-hidden': !mesh.visible }"
              >
                <Checkbox
                  :model-value="mesh.visible"
                  @update:model-value="toggleMeshVisibility(mesh.index, $event === true)"
                />
                <span class="model-preview-object-text">
                  <span class="model-preview-object-name">{{ mesh.name }}</span>
                  <span class="model-preview-object-stats">
                    <span class="model-preview-object-stat">
                      {{ mesh.vertices }}
                      <Share2 class="size-3" aria-hidden="true" />
                    </span>
                    <span class="model-preview-object-stat-separator">/</span>
                    <span class="model-preview-object-stat">
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

    <div class="model-preview-toolbar">
      <button
        type="button"
        class="model-preview-icon-button"
        :title="t('unpack.modelPreviewResetView')"
        @click="resetCamera"
      >
        <RotateCcw class="size-4" />
      </button>
    </div>

    <div v-if="statusText" class="model-preview-status">
      <Loader2 v-if="loading" class="size-4 animate-spin" />
      <AlertTriangle v-else-if="error" class="size-4" />
      <Box v-else class="size-4" />
      <span>{{ statusText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, shallowRef, unref, watch, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  AlertTriangle,
  Box,
  ChevronRight,
  Eye,
  EyeOff,
  Layers,
  Loader2,
  RotateCcw,
  Share2,
  Triangle
} from 'lucide-vue-next'
import { modelInsightLoadMeshAssets, type ModelInsightMeshAssets } from '@/api/tauri/utils'
import { Checkbox } from '@/components/ui/checkbox'
import { Switch } from '@/components/ui/switch'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import type { ExplorerEntry } from '@/lib/unpackExplorer'
import { meshToPreviewModel, type PreviewModel } from '@/lib/modelInsight/wasm'
import {
  loadModelTextureImages,
  loadModelTextureUrls,
  type ModelTextureImages
} from '@/lib/modelInsight/textures'
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

type MeshPreviewGroup = {
  key: string
  name: string
  meshes: MeshPreviewItem[]
}

let glState: ModelPreviewRenderState | null = null
let resizeObserver: ResizeObserver | null = null
let animationFrame = 0
let pointerState: {
  id: number
  x: number
  y: number
  button: number
} | null = null

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
  'model-preview',
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
      meshes: [mesh]
    })
  }
  return Array.from(groups.values())
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
  if (!entry.hash || !entry.belongsTo) {
    error.value = t('unpack.modelInsightMissingSource')
    return
  }

  loading.value = true
  error.value = null

  try {
    const assets = await modelInsightLoadMeshAssets({
      hash: entry.hash,
      belongsTo: entry.belongsTo,
      entryPath: entry.path
    })
    const result = await meshToPreviewModel({
      meshBytes: toUint8Array(assets.meshData),
      meshFileVersion: assets.meshFileVersion,
      mdfBytes: assets.mdfData ? toUint8Array(assets.mdfData) : null,
      mdfFileVersion: assets.mdfFileVersion ?? null
    })
    const textureImages = await loadPreviewTextureImages(assets, result.preview, entry.belongsTo)
    preview.value = result.preview
    resetVisibleMeshes(result.preview)
    buildRenderState(result.preview, textureImages)
    resetCamera()
  } catch (caught) {
    const message = caught instanceof Error ? caught.message : String(caught)
    error.value = message
  } finally {
    loading.value = false
  }
}

async function loadPreviewTextureImages(
  assets: ModelInsightMeshAssets,
  model: PreviewModel,
  belongsTo?: string
) {
  try {
    const textureUrls = await loadModelTextureUrls(assets, model, belongsTo, {
      textureResolution: meshPreviewTextureResolution.value
    })
    return await loadModelTextureImages(textureUrls)
  } catch {
    return {}
  }
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

function toUint8Array(value: number[] | Uint8Array) {
  return value instanceof Uint8Array ? value : Uint8Array.from(value)
}
</script>

<style scoped>
.model-preview {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

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
  display: block;
  width: 100%;
  height: 100%;
  touch-action: none;
  cursor: grab;
}

.model-preview-canvas:active {
  cursor: grabbing;
}

.model-preview-left-ui {
  position: absolute;
  top: 12px;
  left: 12px;
  display: grid;
  width: min(280px, calc(100% - 24px));
  gap: 8px;
}

.model-preview-panel {
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  background: rgba(14, 16, 20, 0.74);
  color: rgba(236, 241, 245, 0.9);
  font-size: 12px;
  line-height: 1.35;
  box-shadow: 0 16px 42px rgba(0, 0, 0, 0.22);
  backdrop-filter: blur(10px);
}

.model-preview-panel-header,
.model-preview-panel-title-button,
.model-preview-control-row,
.model-preview-switch-row,
.model-preview-object-actions,
.model-preview-action-button,
.model-preview-group-row,
.model-preview-object-row {
  display: flex;
  align-items: center;
}

.model-preview-panel-header {
  justify-content: space-between;
  gap: 10px;
  padding: 9px;
}

.model-preview-panel-title-button {
  min-width: 0;
  gap: 6px;
  border-radius: 6px;
  color: rgba(246, 248, 250, 0.94);
  transition:
    background 120ms ease,
    color 120ms ease;
}

.model-preview-panel-title-button:hover {
  color: #ffffff;
}

.model-preview-panel-title {
  min-width: 0;
  overflow: hidden;
  color: rgba(246, 248, 250, 0.94);
  font-size: 11px;
  font-weight: 650;
  letter-spacing: 0;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-preview-chevron {
  flex-shrink: 0;
  transition: transform 120ms ease;
}

.model-preview-chevron-expanded {
  transform: rotate(90deg);
}

.model-preview-settings-body {
  display: grid;
  gap: 8px;
  padding: 0 9px 9px;
}

.model-preview-control-row {
  justify-content: space-between;
  gap: 10px;
}

.model-preview-control-row > span {
  min-width: max-content;
  color: rgba(205, 213, 224, 0.76);
}

.model-preview-segmented {
  display: inline-flex;
  min-width: 0;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.055);
}

.model-preview-segment,
.model-preview-action-button,
.model-preview-group-row {
  color: rgba(224, 230, 238, 0.78);
  transition:
    background 120ms ease,
    color 120ms ease,
    border-color 120ms ease;
}

.model-preview-segment {
  display: inline-flex;
  min-width: 0;
  align-items: center;
  gap: 5px;
  padding: 4px 7px;
  font-size: 11px;
}

.model-preview-segment:hover,
.model-preview-action-button:hover,
.model-preview-group-row:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #ffffff;
}

.model-preview-segment-active {
  background: rgba(255, 255, 255, 0.15);
  color: #ffffff;
}

.model-preview-swatch {
  width: 12px;
  height: 12px;
  flex-shrink: 0;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.26);
}

.model-preview-swatch-dark {
  background: #101215;
}

.model-preview-swatch-light {
  background: #dce1e7;
}

.model-preview-object-actions {
  gap: 6px;
}

.model-preview-action-button {
  width: 26px;
  height: 26px;
  justify-content: center;
  border-radius: 6px;
}

.model-preview-object-list {
  max-height: min(320px, calc(100vh - 280px));
  min-height: 0;
  overflow: auto;
  border-top: 1px solid rgba(255, 255, 255, 0.09);
  padding: 4px;
}

.model-preview-object-group {
  display: grid;
  gap: 2px;
}

.model-preview-group-row {
  min-width: 0;
  width: 100%;
  gap: 6px;
  border-radius: 6px;
  padding: 6px;
  font-size: 12px;
  font-weight: 600;
  text-align: left;
}

.model-preview-group-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-preview-group-children {
  display: grid;
  gap: 2px;
  margin-left: 12px;
  padding-left: 8px;
  border-left: 1px solid rgba(255, 255, 255, 0.1);
}

.model-preview-object-row {
  min-width: 0;
  gap: 8px;
  border-radius: 6px;
  padding: 6px;
  color: rgba(236, 241, 245, 0.9);
  cursor: pointer;
  transition:
    background 120ms ease,
    opacity 120ms ease;
}

.model-preview-object-row:hover {
  background: rgba(255, 255, 255, 0.08);
}

.model-preview-object-row-hidden {
  opacity: 0.55;
}

.model-preview-object-text {
  display: grid;
  min-width: 0;
  gap: 1px;
}

.model-preview-object-name,
.model-preview-object-stats {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-preview-object-name {
  color: rgba(246, 248, 250, 0.94);
  font-size: 12px;
}

.model-preview-object-stats {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: rgba(205, 213, 224, 0.62);
  font-size: 11px;
}

.model-preview-object-stat {
  display: inline-flex;
  align-items: center;
  gap: 3px;
}

.model-preview-object-stat-separator {
  color: rgba(205, 213, 224, 0.42);
}

.model-preview-toolbar {
  position: absolute;
  top: 12px;
  right: 12px;
  display: flex;
  gap: 6px;
  padding: 4px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  background: rgba(14, 16, 20, 0.72);
  backdrop-filter: blur(10px);
}

.model-preview-icon-button {
  display: inline-flex;
  width: 30px;
  height: 30px;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: rgba(236, 241, 245, 0.86);
  transition:
    background 120ms ease,
    color 120ms ease;
}

.model-preview-icon-button:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #ffffff;
}

.model-preview-status {
  position: absolute;
  left: 12px;
  bottom: 12px;
  display: flex;
  max-width: min(520px, calc(100% - 24px));
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  background: rgba(14, 16, 20, 0.72);
  color: rgba(236, 241, 245, 0.9);
  font-size: 12px;
  line-height: 1.35;
  backdrop-filter: blur(10px);
}

.model-preview-status span {
  min-width: 0;
  overflow-wrap: anywhere;
}
</style>
