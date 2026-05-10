import { logFrontendWarn } from '@/utils/frontendLog'

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
  default?: (
    moduleOrPath?: RequestInfo | URL | BufferSource | WebAssembly.Module
  ) => Promise<unknown>
  initModelInsightWasm: () => void
  modelInsightWasmVersion: () => string
  meshToPreviewModel: (
    meshBytes: Uint8Array,
    meshFileVersion: number,
    streamingBufferBytes?: Uint8Array,
    mdfBytes?: Uint8Array,
    mdfFileVersion?: number
  ) => MeshPreviewResponse
}

let wasmPromise: Promise<ModelInsightWasmModule> | null = null
let unavailableWarned = false

export class ModelInsightWasmUnavailableError extends Error {
  cause: unknown

  constructor(cause: unknown) {
    super('Model preview module is unavailable.')
    this.name = 'ModelInsightWasmUnavailableError'
    this.cause = cause
  }
}

export async function loadModelInsightWasm(): Promise<ModelInsightWasmModule> {
  wasmPromise ??= importModelInsightWasm()
  return wasmPromise
}

export async function meshToPreviewModel(options: {
  meshBytes: Uint8Array
  meshFileVersion: number
  streamingBufferBytes?: Uint8Array | null
  mdfBytes?: Uint8Array | null
  mdfFileVersion?: number | null
}): Promise<MeshPreviewResponse> {
  const wasm = await loadModelInsightWasm()
  return wasm.meshToPreviewModel(
    options.meshBytes,
    options.meshFileVersion,
    options.streamingBufferBytes ?? undefined,
    options.mdfBytes ?? undefined,
    options.mdfFileVersion ?? undefined
  )
}

async function importModelInsightWasm(): Promise<ModelInsightWasmModule> {
  const wasmBasePath = '../../wasm/model-insight'
  const moduleUrl = new URL(`${wasmBasePath}/model_insight.js`, import.meta.url).href
  const wasmUrl = new URL(`${wasmBasePath}/model_insight_bg.wasm`, import.meta.url).href
  try {
    const wasm = (await import(/* @vite-ignore */ moduleUrl)) as ModelInsightWasmModule
    if (wasm.default) {
      await wasm.default(wasmUrl)
    }
    wasm.initModelInsightWasm()
    return wasm
  } catch (error) {
    warnUnavailableOnce(error)
    throw new ModelInsightWasmUnavailableError(error)
  }
}

export function isModelInsightWasmUnavailableError(
  error: unknown
): error is ModelInsightWasmUnavailableError {
  return error instanceof ModelInsightWasmUnavailableError
}

function warnUnavailableOnce(error: unknown) {
  if (unavailableWarned) return
  unavailableWarned = true
  logFrontendWarn(
    'modelInsight.wasm',
    `optional model preview module is unavailable error=${error instanceof Error ? error.message : String(error)}`
  )
}
