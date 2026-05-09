import {
  modelInsightLoadTexturePreviews,
  type ModelInsightMeshAssets,
  type ModelTextureResolution
} from '@/api/tauri/utils'
import type { PreviewModel } from './wasm'

export type ModelTextureUrls = Record<string, string>
export type ModelTextureImages = Record<string, HTMLImageElement>
export type ModelTextureLoadOptions = {
  textureResolution?: ModelTextureResolution
}

export async function loadModelTextureUrls(
  assets: ModelInsightMeshAssets,
  model: PreviewModel,
  belongsTo?: string,
  options: ModelTextureLoadOptions = {}
): Promise<ModelTextureUrls> {
  const texturePaths = collectAlbedoTexturePaths(model)
  const baseEntryPath = assets.mdfEntryPath ?? assets.meshEntryPath
  if (!baseEntryPath || texturePaths.length === 0) return {}

  const previews = await modelInsightLoadTexturePreviews({
    belongsTo,
    baseEntryPath,
    texturePaths,
    textureResolution: options.textureResolution
  })

  return Object.fromEntries(
    previews.map((preview) => [
      preview.texturePath,
      URL.createObjectURL(
        new Blob([toUint8Array(preview.previewData).buffer as ArrayBuffer], { type: 'image/png' })
      )
    ])
  )
}

export async function loadModelTextureImages(
  textureUrls: ModelTextureUrls
): Promise<ModelTextureImages> {
  const entries = await Promise.all(
    Object.entries(textureUrls).map(async ([texturePath, url]) => {
      try {
        return [texturePath, await loadImage(url)] as const
      } catch {
        return null
      } finally {
        URL.revokeObjectURL(url)
      }
    })
  )

  return Object.fromEntries(entries.filter((entry) => entry !== null))
}

function collectAlbedoTexturePaths(model: PreviewModel) {
  return Array.from(
    new Set(
      model.materials
        .map((material) => material.albedoTexturePath)
        .filter((path): path is string => Boolean(path))
    )
  )
}

function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image()
    image.onload = () => resolve(image)
    image.onerror = () => reject(new Error(`Failed to load model texture: ${url}`))
    image.src = url
  })
}

function toUint8Array(value: number[] | Uint8Array) {
  return value instanceof Uint8Array ? value : Uint8Array.from(value)
}
