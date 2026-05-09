<template>
  <div ref="rootRef" class="model-preview">
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
import { computed, onMounted, onUnmounted, ref, shallowRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { AlertTriangle, Box, Loader2, RotateCcw } from 'lucide-vue-next'
import { modelInsightLoadMeshAssets, type ModelInsightMeshAssets } from '@/api/tauri/utils'
import type { ExplorerEntry } from '@/lib/unpackExplorer'
import { meshToPreviewModel, type PreviewModel, type PreviewSubmesh } from '@/lib/modelInsight/wasm'

const props = defineProps<{
  entry: ExplorerEntry
}>()

const { t } = useI18n()
const rootRef = ref<HTMLDivElement | null>(null)
const canvasRef = ref<HTMLCanvasElement | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const preview = shallowRef<PreviewModel | null>(null)

let glState: RenderState | null = null
let resizeObserver: ResizeObserver | null = null
let animationFrame = 0
let pointerState: {
  id: number
  x: number
  y: number
  button: number
} | null = null

const camera = {
  target: [0, 0, 0] as Vec3,
  yaw: -0.65,
  pitch: -0.28,
  distance: 1
}

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
    preview.value = result.preview
    buildRenderState(result.preview)
    resetCamera()
  } catch (caught) {
    const message = caught instanceof Error ? caught.message : String(caught)
    error.value = message
  } finally {
    loading.value = false
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
    glState = createRenderState(gl)
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught)
    return
  }
  resizeObserver = new ResizeObserver(() => resizeCanvas())
  resizeObserver.observe(root)
  resizeCanvas()
  renderLoop()
}

function buildRenderState(model: PreviewModel) {
  if (!glState) return
  const gl = glState.gl
  for (const mesh of glState.meshes) {
    gl.deleteBuffer(mesh.positions)
    gl.deleteBuffer(mesh.normals)
    gl.deleteBuffer(mesh.indices)
  }
  glState.meshes = model.meshes
    .filter((mesh) => mesh.positions.length > 0 && mesh.indices.length > 0)
    .map((mesh) => createRenderableMesh(gl, model, mesh))
}

function resetCamera() {
  const model = preview.value
  if (!model) return
  const radius = Number.isFinite(model.bounds.sphereRadius)
    ? Math.max(model.bounds.sphereRadius, 0.5)
    : 0.5
  camera.target = [...model.bounds.sphereCenter] as Vec3
  camera.yaw = -0.65
  camera.pitch = -0.28
  camera.distance = radius * 2.6
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
  const view = cameraVectors()
  const scale = camera.distance * 0.0015
  camera.target = add3(camera.target, scale3(view.right, -dx * scale))
  camera.target = add3(camera.target, scale3(view.up, dy * scale))
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
  const gl = state.gl
  const canvas = gl.canvas as HTMLCanvasElement
  resizeCanvas()

  gl.clearColor(0.02, 0.023, 0.028, 0)
  gl.clearDepth(1)
  gl.enable(gl.DEPTH_TEST)
  gl.enable(gl.CULL_FACE)
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT)

  const aspect = canvas.width / Math.max(canvas.height, 1)
  const projection = perspective(Math.PI / 4, aspect, 0.01, Math.max(camera.distance * 20, 100))
  const vectors = cameraVectors()
  const eye = add3(camera.target, scale3(vectors.forward, -camera.distance))
  const view = lookAt(eye, camera.target, [0, 1, 0])

  gl.useProgram(state.program)
  gl.uniformMatrix4fv(state.uniforms.projection, false, projection)
  gl.uniformMatrix4fv(state.uniforms.view, false, view)
  gl.uniform3f(state.uniforms.lightDir, 0.45, 0.75, 0.35)

  for (const mesh of state.meshes) {
    gl.bindBuffer(gl.ARRAY_BUFFER, mesh.positions)
    gl.enableVertexAttribArray(state.attributes.position)
    gl.vertexAttribPointer(state.attributes.position, 3, gl.FLOAT, false, 0, 0)

    gl.bindBuffer(gl.ARRAY_BUFFER, mesh.normals)
    gl.enableVertexAttribArray(state.attributes.normal)
    gl.vertexAttribPointer(state.attributes.normal, 3, gl.FLOAT, false, 0, 0)

    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, mesh.indices)
    gl.uniform3fv(state.uniforms.color, mesh.color)
    gl.drawElements(gl.TRIANGLES, mesh.indexCount, gl.UNSIGNED_INT, 0)
  }
}

function createRenderState(gl: WebGLRenderingContext): RenderState {
  const extension = gl.getExtension('OES_element_index_uint')
  if (!extension) {
    throw new Error(t('unpack.modelPreviewIndexUnavailable'))
  }

  const program = createProgram(gl, VERTEX_SHADER, FRAGMENT_SHADER)
  const attributes = {
    position: gl.getAttribLocation(program, 'aPosition'),
    normal: gl.getAttribLocation(program, 'aNormal')
  }
  const uniforms = {
    projection: requiredUniform(gl, program, 'uProjection'),
    view: requiredUniform(gl, program, 'uView'),
    color: requiredUniform(gl, program, 'uColor'),
    lightDir: requiredUniform(gl, program, 'uLightDir')
  }

  return {
    gl,
    program,
    attributes,
    uniforms,
    meshes: []
  }
}

function createRenderableMesh(
  gl: WebGLRenderingContext,
  model: PreviewModel,
  mesh: PreviewSubmesh
): RenderableMesh {
  const positions = gl.createBuffer()
  const normals = gl.createBuffer()
  const indices = gl.createBuffer()
  if (!positions || !normals || !indices) {
    throw new Error(t('unpack.modelPreviewBufferFailed'))
  }

  gl.bindBuffer(gl.ARRAY_BUFFER, positions)
  gl.bufferData(gl.ARRAY_BUFFER, flatten3(mesh.positions), gl.STATIC_DRAW)

  gl.bindBuffer(gl.ARRAY_BUFFER, normals)
  gl.bufferData(gl.ARRAY_BUFFER, flatten3(validNormals(mesh)), gl.STATIC_DRAW)

  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indices)
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint32Array(mesh.indices), gl.STATIC_DRAW)

  const material = model.materials[mesh.materialIndex]
  return {
    positions,
    normals,
    indices,
    indexCount: mesh.indices.length,
    color: materialColor(material?.name ?? mesh.name)
  }
}

function disposeRenderState() {
  const state = glState
  if (!state) return
  for (const mesh of state.meshes) {
    state.gl.deleteBuffer(mesh.positions)
    state.gl.deleteBuffer(mesh.normals)
    state.gl.deleteBuffer(mesh.indices)
  }
  state.gl.deleteProgram(state.program)
  glState = null
}

function validNormals(mesh: PreviewSubmesh) {
  if (mesh.normals.length === mesh.positions.length) return mesh.normals
  return mesh.positions.map(() => [0, 1, 0] as [number, number, number])
}

function flatten3(values: Array<[number, number, number]>) {
  const result = new Float32Array(values.length * 3)
  values.forEach((value, index) => {
    result[index * 3] = value[0]
    result[index * 3 + 1] = value[1]
    result[index * 3 + 2] = value[2]
  })
  return result
}

function materialColor(name: string): Float32Array {
  let hash = 2166136261
  for (let index = 0; index < name.length; index += 1) {
    hash ^= name.charCodeAt(index)
    hash = Math.imul(hash, 16777619)
  }
  const hue = (hash >>> 0) % 360
  return new Float32Array(hslToRgb(hue / 360, 0.45, 0.62))
}

function hslToRgb(hue: number, saturation: number, lightness: number): Vec3 {
  if (saturation === 0) return [lightness, lightness, lightness]
  const q =
    lightness < 0.5 ? lightness * (1 + saturation) : lightness + saturation - lightness * saturation
  const p = 2 * lightness - q
  return [hueToRgb(p, q, hue + 1 / 3), hueToRgb(p, q, hue), hueToRgb(p, q, hue - 1 / 3)]
}

function hueToRgb(p: number, q: number, value: number) {
  let t = value
  if (t < 0) t += 1
  if (t > 1) t -= 1
  if (t < 1 / 6) return p + (q - p) * 6 * t
  if (t < 1 / 2) return q
  if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6
  return p
}

function createProgram(gl: WebGLRenderingContext, vertexSource: string, fragmentSource: string) {
  const vertex = compileShader(gl, gl.VERTEX_SHADER, vertexSource)
  const fragment = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSource)
  const program = gl.createProgram()
  if (!program) throw new Error(t('unpack.modelPreviewProgramFailed'))
  gl.attachShader(program, vertex)
  gl.attachShader(program, fragment)
  gl.linkProgram(program)
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    const detail = gl.getProgramInfoLog(program) ?? 'unknown link error'
    gl.deleteProgram(program)
    throw new Error(detail)
  }
  gl.deleteShader(vertex)
  gl.deleteShader(fragment)
  return program
}

function compileShader(gl: WebGLRenderingContext, type: number, source: string) {
  const shader = gl.createShader(type)
  if (!shader) throw new Error(t('unpack.modelPreviewShaderFailed'))
  gl.shaderSource(shader, source)
  gl.compileShader(shader)
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const detail = gl.getShaderInfoLog(shader) ?? 'unknown shader error'
    gl.deleteShader(shader)
    throw new Error(detail)
  }
  return shader
}

function requiredUniform(gl: WebGLRenderingContext, program: WebGLProgram, name: string) {
  const uniform = gl.getUniformLocation(program, name)
  if (!uniform) throw new Error(`missing shader uniform ${name}`)
  return uniform
}

function cameraVectors() {
  const cosPitch = Math.cos(camera.pitch)
  const forward: Vec3 = [
    Math.sin(camera.yaw) * cosPitch,
    Math.sin(camera.pitch),
    Math.cos(camera.yaw) * cosPitch
  ]
  const right = normalize3(cross3(forward, [0, 1, 0]))
  const up = normalize3(cross3(right, forward))
  return { forward, right, up }
}

function perspective(fovy: number, aspect: number, near: number, far: number) {
  const f = 1 / Math.tan(fovy / 2)
  const out = new Float32Array(16)
  out[0] = f / aspect
  out[5] = f
  out[10] = (far + near) / (near - far)
  out[11] = -1
  out[14] = (2 * far * near) / (near - far)
  return out
}

function lookAt(eye: Vec3, target: Vec3, up: Vec3) {
  const z = normalize3(sub3(eye, target))
  const x = normalize3(cross3(up, z))
  const y = cross3(z, x)
  const out = new Float32Array(16)
  out[0] = x[0]
  out[1] = y[0]
  out[2] = z[0]
  out[4] = x[1]
  out[5] = y[1]
  out[6] = z[1]
  out[8] = x[2]
  out[9] = y[2]
  out[10] = z[2]
  out[12] = -dot3(x, eye)
  out[13] = -dot3(y, eye)
  out[14] = -dot3(z, eye)
  out[15] = 1
  return out
}

function add3(a: Vec3, b: Vec3): Vec3 {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

function sub3(a: Vec3, b: Vec3): Vec3 {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

function scale3(value: Vec3, scalar: number): Vec3 {
  return [value[0] * scalar, value[1] * scalar, value[2] * scalar]
}

function cross3(a: Vec3, b: Vec3): Vec3 {
  return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

function dot3(a: Vec3, b: Vec3) {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

function normalize3(value: Vec3): Vec3 {
  const length = Math.hypot(value[0], value[1], value[2]) || 1
  return [value[0] / length, value[1] / length, value[2] / length]
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max)
}

function toUint8Array(value: number[] | Uint8Array) {
  return value instanceof Uint8Array ? value : Uint8Array.from(value)
}

type Vec3 = [number, number, number]

interface RenderableMesh {
  positions: WebGLBuffer
  normals: WebGLBuffer
  indices: WebGLBuffer
  indexCount: number
  color: Float32Array
}

interface RenderState {
  gl: WebGLRenderingContext
  program: WebGLProgram
  attributes: {
    position: number
    normal: number
  }
  uniforms: {
    projection: WebGLUniformLocation
    view: WebGLUniformLocation
    color: WebGLUniformLocation
    lightDir: WebGLUniformLocation
  }
  meshes: RenderableMesh[]
}

const VERTEX_SHADER = `
attribute vec3 aPosition;
attribute vec3 aNormal;
uniform mat4 uProjection;
uniform mat4 uView;
varying vec3 vNormal;

void main() {
  vNormal = aNormal;
  gl_Position = uProjection * uView * vec4(aPosition, 1.0);
}
`

const FRAGMENT_SHADER = `
precision mediump float;
uniform vec3 uColor;
uniform vec3 uLightDir;
varying vec3 vNormal;

void main() {
  float light = max(dot(normalize(vNormal), normalize(uLightDir)), 0.0) * 0.65 + 0.35;
  gl_FragColor = vec4(uColor * light, 1.0);
}
`
</script>

<style scoped>
.model-preview {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background:
    linear-gradient(90deg, rgba(255, 255, 255, 0.035) 1px, transparent 1px),
    linear-gradient(0deg, rgba(255, 255, 255, 0.035) 1px, transparent 1px), #101215;
  background-size: 28px 28px;
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
