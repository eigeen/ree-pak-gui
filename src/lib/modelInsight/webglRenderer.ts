import type { ModelTextureImages } from './textures'
import type { PreviewModel, PreviewSubmesh } from './wasm'

export interface ModelPreviewRenderMessages {
  indexUnavailable?: string
  bufferFailed?: string
  programFailed?: string
  shaderFailed?: string
}

export interface ModelPreviewCameraOptions {
  cameraYaw?: number
  cameraPitch?: number
  cameraDistanceScale?: number
  frameY?: number
}

export interface ModelPreviewCamera {
  target: Vec3
  yaw: number
  pitch: number
  distance: number
}

export interface ModelPreviewRenderState {
  gl: WebGLRenderingContext
  program: WebGLProgram
  attributes: {
    position: number
    normal: number
    uv: number
  }
  uniforms: {
    projection: WebGLUniformLocation
    view: WebGLUniformLocation
    color: WebGLUniformLocation
    lightDir: WebGLUniformLocation
    texture: WebGLUniformLocation
    useTexture: WebGLUniformLocation
  }
  meshes: RenderableMesh[]
  messages: Required<ModelPreviewRenderMessages>
}

export type Vec3 = [number, number, number]

const DEFAULT_CAMERA_YAW = -0.65
const DEFAULT_CAMERA_PITCH = -0.48
const DEFAULT_CAMERA_DISTANCE_SCALE = 2.1

const DEFAULT_MESSAGES: Required<ModelPreviewRenderMessages> = {
  indexUnavailable: 'This WebView does not support 32-bit mesh indices.',
  bufferFailed: 'Failed to create model render buffers.',
  programFailed: 'Failed to create model render program.',
  shaderFailed: 'Failed to create model render shader.'
}

export function createModelPreviewRenderState(
  gl: WebGLRenderingContext,
  messages: ModelPreviewRenderMessages = {}
): ModelPreviewRenderState {
  const mergedMessages = { ...DEFAULT_MESSAGES, ...messages }
  const extension = gl.getExtension('OES_element_index_uint')
  if (!extension) {
    throw new Error(mergedMessages.indexUnavailable)
  }

  const program = createProgram(gl, VERTEX_SHADER, FRAGMENT_SHADER, mergedMessages)
  const attributes = {
    position: gl.getAttribLocation(program, 'aPosition'),
    normal: gl.getAttribLocation(program, 'aNormal'),
    uv: gl.getAttribLocation(program, 'aUv')
  }
  const uniforms = {
    projection: requiredUniform(gl, program, 'uProjection'),
    view: requiredUniform(gl, program, 'uView'),
    color: requiredUniform(gl, program, 'uColor'),
    lightDir: requiredUniform(gl, program, 'uLightDir'),
    texture: requiredUniform(gl, program, 'uTexture'),
    useTexture: requiredUniform(gl, program, 'uUseTexture')
  }

  return {
    gl,
    program,
    attributes,
    uniforms,
    meshes: [],
    messages: mergedMessages
  }
}

export function setModelPreviewRenderMeshes(
  state: ModelPreviewRenderState,
  model: PreviewModel,
  textureImages: ModelTextureImages = {}
) {
  disposeModelPreviewRenderMeshes(state)
  state.meshes = model.meshes
    .map((mesh, sourceIndex) => ({ mesh, sourceIndex }))
    .filter(({ mesh }) => mesh.positions.length > 0 && mesh.indices.length > 0)
    .map(({ mesh, sourceIndex }) =>
      createRenderableMesh(state, model, mesh, textureImages, sourceIndex)
    )
}

export function setModelPreviewMeshVisibility(
  state: ModelPreviewRenderState,
  visibleIndexes: Iterable<number>
) {
  const visibleSet = new Set(visibleIndexes)
  for (const mesh of state.meshes) {
    mesh.visible = visibleSet.has(mesh.sourceIndex)
  }
}

export function renderModelPreviewFrame(
  state: ModelPreviewRenderState,
  camera: ModelPreviewCamera
) {
  const gl = state.gl
  const canvas = gl.canvas as HTMLCanvasElement

  gl.viewport(0, 0, canvas.width, canvas.height)
  gl.clearColor(0.02, 0.023, 0.028, 0)
  gl.clearDepth(1)
  gl.enable(gl.DEPTH_TEST)
  gl.disable(gl.CULL_FACE)
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT)

  const aspect = canvas.width / Math.max(canvas.height, 1)
  const projection = perspective(Math.PI / 4, aspect, 0.01, Math.max(camera.distance * 20, 100))
  const vectors = modelPreviewCameraVectors(camera)
  const eye = addVec3(camera.target, scaleVec3(vectors.forward, -camera.distance))
  const view = lookAt(eye, camera.target, [0, 1, 0])

  gl.useProgram(state.program)
  gl.uniformMatrix4fv(state.uniforms.projection, false, projection)
  gl.uniformMatrix4fv(state.uniforms.view, false, view)
  gl.uniform3f(state.uniforms.lightDir, 0.45, 0.75, 0.35)

  for (const mesh of state.meshes) {
    if (!mesh.visible) continue

    gl.bindBuffer(gl.ARRAY_BUFFER, mesh.positions)
    gl.enableVertexAttribArray(state.attributes.position)
    gl.vertexAttribPointer(state.attributes.position, 3, gl.FLOAT, false, 0, 0)

    gl.bindBuffer(gl.ARRAY_BUFFER, mesh.normals)
    gl.enableVertexAttribArray(state.attributes.normal)
    gl.vertexAttribPointer(state.attributes.normal, 3, gl.FLOAT, false, 0, 0)

    gl.bindBuffer(gl.ARRAY_BUFFER, mesh.uvs)
    gl.enableVertexAttribArray(state.attributes.uv)
    gl.vertexAttribPointer(state.attributes.uv, 2, gl.FLOAT, false, 0, 0)

    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, mesh.indices)
    gl.uniform3fv(state.uniforms.color, mesh.color)
    gl.uniform1i(state.uniforms.useTexture, mesh.texture ? 1 : 0)
    if (mesh.texture) {
      gl.activeTexture(gl.TEXTURE0)
      gl.bindTexture(gl.TEXTURE_2D, mesh.texture)
      gl.uniform1i(state.uniforms.texture, 0)
    }
    gl.drawElements(gl.TRIANGLES, mesh.indexCount, gl.UNSIGNED_INT, 0)
  }
}

export function createDefaultModelPreviewCamera(
  model: PreviewModel,
  options: ModelPreviewCameraOptions = {}
): ModelPreviewCamera {
  const radius = Number.isFinite(model.bounds.sphereRadius)
    ? Math.max(model.bounds.sphereRadius, 0.5)
    : 0.5
  const camera: ModelPreviewCamera = {
    target: [...model.bounds.sphereCenter] as Vec3,
    yaw: options.cameraYaw ?? DEFAULT_CAMERA_YAW,
    pitch: options.cameraPitch ?? DEFAULT_CAMERA_PITCH,
    distance: radius * (options.cameraDistanceScale ?? DEFAULT_CAMERA_DISTANCE_SCALE)
  }
  const frameY = options.frameY ?? 0
  if (frameY !== 0) {
    camera.target = subVec3(
      camera.target,
      scaleVec3(modelPreviewCameraVectors(camera).up, radius * frameY)
    )
  }
  return camera
}

export function modelPreviewCameraVectors(camera: ModelPreviewCamera) {
  const cosPitch = Math.cos(camera.pitch)
  const forward: Vec3 = [
    Math.sin(camera.yaw) * cosPitch,
    Math.sin(camera.pitch),
    Math.cos(camera.yaw) * cosPitch
  ]
  const right = normalizeVec3(crossVec3(forward, [0, 1, 0]))
  const up = normalizeVec3(crossVec3(right, forward))
  return { forward, right, up }
}

export function addVec3(a: Vec3, b: Vec3): Vec3 {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

export function scaleVec3(value: Vec3, scalar: number): Vec3 {
  return [value[0] * scalar, value[1] * scalar, value[2] * scalar]
}

export function disposeModelPreviewRenderState(state: ModelPreviewRenderState) {
  disposeModelPreviewRenderMeshes(state)
  state.gl.deleteProgram(state.program)
}

export function disposeModelPreviewRenderMeshes(state: ModelPreviewRenderState) {
  for (const mesh of state.meshes) {
    state.gl.deleteBuffer(mesh.positions)
    state.gl.deleteBuffer(mesh.normals)
    state.gl.deleteBuffer(mesh.uvs)
    state.gl.deleteBuffer(mesh.indices)
    if (mesh.texture) state.gl.deleteTexture(mesh.texture)
  }
  state.meshes = []
}

function createRenderableMesh(
  state: ModelPreviewRenderState,
  model: PreviewModel,
  mesh: PreviewSubmesh,
  textureImages: ModelTextureImages,
  sourceIndex: number
): RenderableMesh {
  const gl = state.gl
  const positions = gl.createBuffer()
  const normals = gl.createBuffer()
  const uvs = gl.createBuffer()
  const indices = gl.createBuffer()
  if (!positions || !normals || !uvs || !indices) {
    throw new Error(state.messages.bufferFailed)
  }

  gl.bindBuffer(gl.ARRAY_BUFFER, positions)
  gl.bufferData(gl.ARRAY_BUFFER, flatten3(mesh.positions), gl.STATIC_DRAW)

  gl.bindBuffer(gl.ARRAY_BUFFER, normals)
  gl.bufferData(gl.ARRAY_BUFFER, flatten3(validNormals(mesh)), gl.STATIC_DRAW)

  gl.bindBuffer(gl.ARRAY_BUFFER, uvs)
  gl.bufferData(gl.ARRAY_BUFFER, flatten2(validUvs(mesh)), gl.STATIC_DRAW)

  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indices)
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint32Array(mesh.indices), gl.STATIC_DRAW)

  const material = model.materials[mesh.materialIndex]
  const textureImage = material?.albedoTexturePath
    ? textureImages[material.albedoTexturePath]
    : null
  return {
    sourceIndex,
    positions,
    normals,
    uvs,
    indices,
    indexCount: mesh.indices.length,
    color: materialColor(material?.name ?? mesh.name),
    texture: textureImage ? createTexture(gl, textureImage) : null,
    visible: true
  }
}

function validNormals(mesh: PreviewSubmesh) {
  if (mesh.normals.length === mesh.positions.length) return mesh.normals
  return mesh.positions.map(() => [0, 1, 0] as Vec3)
}

function validUvs(mesh: PreviewSubmesh) {
  if (mesh.uvs.length === mesh.positions.length) return mesh.uvs
  return mesh.positions.map(() => [0, 0] as [number, number])
}

function flatten3(values: Vec3[]) {
  const result = new Float32Array(values.length * 3)
  values.forEach((value, index) => {
    result[index * 3] = value[0]
    result[index * 3 + 1] = value[1]
    result[index * 3 + 2] = value[2]
  })
  return result
}

function flatten2(values: Array<[number, number]>) {
  const result = new Float32Array(values.length * 2)
  values.forEach((value, index) => {
    result[index * 2] = value[0]
    result[index * 2 + 1] = value[1]
  })
  return result
}

function createTexture(gl: WebGLRenderingContext, image: HTMLImageElement) {
  const texture = gl.createTexture()
  if (!texture) return null
  gl.bindTexture(gl.TEXTURE_2D, texture)
  gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false)
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image)
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE)
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE)
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR)
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR)
  gl.bindTexture(gl.TEXTURE_2D, null)
  return texture
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

function createProgram(
  gl: WebGLRenderingContext,
  vertexSource: string,
  fragmentSource: string,
  messages: Required<ModelPreviewRenderMessages>
) {
  const vertex = compileShader(gl, gl.VERTEX_SHADER, vertexSource, messages)
  const fragment = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSource, messages)
  const program = gl.createProgram()
  if (!program) throw new Error(messages.programFailed)
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

function compileShader(
  gl: WebGLRenderingContext,
  type: number,
  source: string,
  messages: Required<ModelPreviewRenderMessages>
) {
  const shader = gl.createShader(type)
  if (!shader) throw new Error(messages.shaderFailed)
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
  const z = normalizeVec3(subVec3(eye, target))
  const x = normalizeVec3(crossVec3(up, z))
  const y = crossVec3(z, x)
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
  out[12] = -dotVec3(x, eye)
  out[13] = -dotVec3(y, eye)
  out[14] = -dotVec3(z, eye)
  out[15] = 1
  return out
}

function subVec3(a: Vec3, b: Vec3): Vec3 {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

function crossVec3(a: Vec3, b: Vec3): Vec3 {
  return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

function dotVec3(a: Vec3, b: Vec3) {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

function normalizeVec3(value: Vec3): Vec3 {
  const length = Math.hypot(value[0], value[1], value[2]) || 1
  return [value[0] / length, value[1] / length, value[2] / length]
}

interface RenderableMesh {
  sourceIndex: number
  positions: WebGLBuffer
  normals: WebGLBuffer
  uvs: WebGLBuffer
  indices: WebGLBuffer
  indexCount: number
  color: Float32Array
  texture: WebGLTexture | null
  visible: boolean
}

const VERTEX_SHADER = `
attribute vec3 aPosition;
attribute vec3 aNormal;
attribute vec2 aUv;
uniform mat4 uProjection;
uniform mat4 uView;
varying vec3 vNormal;
varying vec2 vUv;

void main() {
  vNormal = aNormal;
  vUv = aUv;
  gl_Position = uProjection * uView * vec4(aPosition, 1.0);
}
`

const FRAGMENT_SHADER = `
precision mediump float;
uniform vec3 uColor;
uniform vec3 uLightDir;
uniform sampler2D uTexture;
uniform bool uUseTexture;
varying vec3 vNormal;
varying vec2 vUv;

void main() {
  float light = max(dot(normalize(vNormal), normalize(uLightDir)), 0.0) * 0.65 + 0.35;
  vec3 baseColor = uUseTexture ? texture2D(uTexture, vUv).rgb : uColor;
  gl_FragColor = vec4(baseColor * light, 1.0);
}
`
