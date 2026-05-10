import type { JsSafeHash, PakId } from '@/api/tauri/pak'
import {
  modelInsightLoadMeshAssets,
  type ModelInsightMeshAssets,
  type ModelTextureResolution
} from '@/api/tauri/utils'
import { toUint8Array } from './bytes'
import { loadModelTextureImages, loadModelTextureUrls, type ModelTextureImages } from './textures'
import { meshToPreviewModel, type PreviewModel } from './wasm'

export type ModelPreviewSource = {
  hash?: JsSafeHash
  belongsTo?: PakId
  path: string
}

export type ModelPreviewGeometry = {
  assets: ModelInsightMeshAssets
  preview: PreviewModel
}

export type ModelPreviewTextureOptions = {
  textureResolution?: ModelTextureResolution
  warnScope?: string
}

const geometryCache = new Map<string, Promise<ModelPreviewGeometry>>()
const textureCache = new Map<string, Promise<ModelTextureImages>>()

export function clearModelPreviewLoaderCache() {
  geometryCache.clear()
  textureCache.clear()
}

export async function loadModelPreviewGeometry(
  source: ModelPreviewSource
): Promise<ModelPreviewGeometry> {
  const key = modelPreviewSourceKey(source)
  let cached = geometryCache.get(key)
  if (!cached) {
    cached = loadUncachedModelPreviewGeometry(source)
    geometryCache.set(key, cached)
  }

  try {
    return await cached
  } catch (error) {
    geometryCache.delete(key)
    throw error
  }
}

export async function loadModelPreviewTextureImages(
  source: ModelPreviewSource,
  geometry: ModelPreviewGeometry,
  options: ModelPreviewTextureOptions = {}
): Promise<ModelTextureImages> {
  const resolution = options.textureResolution ?? 'standard'
  const key = `${modelPreviewSourceKey(source)}|textures:${resolution}`
  let cached = textureCache.get(key)
  if (!cached) {
    cached = loadUncachedModelPreviewTextureImages(source, geometry, {
      ...options,
      textureResolution: resolution
    })
    textureCache.set(key, cached)
  }

  try {
    return await cached
  } catch (error) {
    textureCache.delete(key)
    throw error
  }
}

function modelPreviewSourceKey(source: ModelPreviewSource) {
  const hash = source.hash?.join(':') ?? 'no-hash'
  return `${source.belongsTo ?? 'no-pak'}|${hash}|${normalizeEntryPath(source.path)}`
}

async function loadUncachedModelPreviewGeometry(
  source: ModelPreviewSource
): Promise<ModelPreviewGeometry> {
  if (!source.hash) {
    throw new Error('Missing source pak or file hash.')
  }

  const assets = await modelInsightLoadMeshAssets({
    hash: source.hash,
    belongsTo: source.belongsTo,
    entryPath: source.path
  })
  const result = await meshToPreviewModel({
    meshBytes: toUint8Array(assets.meshData),
    meshFileVersion: assets.meshFileVersion,
    streamingBufferBytes: assets.streamingBufferData
      ? toUint8Array(assets.streamingBufferData)
      : null,
    mdfBytes: assets.mdfData ? toUint8Array(assets.mdfData) : null,
    mdfFileVersion: assets.mdfFileVersion ?? null
  })

  return {
    assets,
    preview: result.preview
  }
}

async function loadUncachedModelPreviewTextureImages(
  source: ModelPreviewSource,
  geometry: ModelPreviewGeometry,
  options: Required<Pick<ModelPreviewTextureOptions, 'textureResolution'>> &
    Omit<ModelPreviewTextureOptions, 'textureResolution'>
) {
  const textureUrls = await loadModelTextureUrls(
    geometry.assets,
    geometry.preview,
    source.belongsTo,
    {
      textureResolution: options.textureResolution,
      warnScope: options.warnScope,
      warnBasePath: geometry.assets.mdfEntryPath ?? geometry.assets.meshEntryPath
    }
  )
  return await loadModelTextureImages(textureUrls, {
    warnScope: options.warnScope,
    warnBasePath: geometry.assets.mdfEntryPath ?? geometry.assets.meshEntryPath
  })
}

function normalizeEntryPath(path: string) {
  return path.replaceAll('\\', '/').replace(/^\/+/, '')
}
