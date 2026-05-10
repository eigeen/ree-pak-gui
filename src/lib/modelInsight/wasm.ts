export interface PreviewModel {
  meshes: PreviewSubmesh[]
  materials: PreviewMaterial[]
  bounds: PreviewBounds
}

export interface PreviewSubmesh {
  name: string
  materialIndex: number
  positions: Array<[number, number, number]>
  normals: Array<[number, number, number]>
  uvs: Array<[number, number]>
  indices: number[]
}

export interface PreviewMaterial {
  name: string
  albedoTexturePath?: string | null
  normalTexturePath?: string | null
}

export interface PreviewBounds {
  min: [number, number, number]
  max: [number, number, number]
  sphereCenter: [number, number, number]
  sphereRadius: number
}

export interface MeshPreviewResponse {
  meshFileVersion: number
  mdfFileVersion?: number | null
  preview: PreviewModel
}

interface ModelInsightWasmModule {
  default?: (moduleOrPath?: RequestInfo | URL | BufferSource | WebAssembly.Module) => Promise<unknown>
  initModelInsightWasm: () => void
  modelInsightWasmVersion: () => string
  meshToPreviewModel: (
    meshBytes: Uint8Array,
    meshFileVersion: number,
    mdfBytes?: Uint8Array,
    mdfFileVersion?: number
  ) => MeshPreviewResponse
  texToDds?: (texBytes: Uint8Array, mipmapCount?: number) => Uint8Array
}

let wasmPromise: Promise<ModelInsightWasmModule> | null = null

export async function loadModelInsightWasm(): Promise<ModelInsightWasmModule> {
  wasmPromise ??= importModelInsightWasm()
  return wasmPromise
}

export async function meshToPreviewModel(options: {
  meshBytes: Uint8Array
  meshFileVersion: number
  mdfBytes?: Uint8Array | null
  mdfFileVersion?: number | null
}): Promise<MeshPreviewResponse> {
  const wasm = await loadModelInsightWasm()
  return wasm.meshToPreviewModel(
    options.meshBytes,
    options.meshFileVersion,
    options.mdfBytes ?? undefined,
    options.mdfFileVersion ?? undefined
  )
}

export async function texToDds(
  texBytes: Uint8Array,
  mipmapCount?: number
): Promise<Uint8Array> {
  const wasm = await loadModelInsightWasm()
  if (!wasm.texToDds) {
    throw new Error('model-insight wasm was built without TEX to DDS support')
  }
  return wasm.texToDds(texBytes, mipmapCount)
}

async function importModelInsightWasm(): Promise<ModelInsightWasmModule> {
  const moduleUrl = new URL('../../wasm/model-insight/model_insight.js', import.meta.url).href
  const wasmUrl = new URL('../../wasm/model-insight/model_insight_bg.wasm', import.meta.url).href
  const wasm = (await import(/* @vite-ignore */ moduleUrl)) as ModelInsightWasmModule
  if (wasm.default) {
    await wasm.default(wasmUrl)
  }
  wasm.initModelInsightWasm()
  return wasm
}
