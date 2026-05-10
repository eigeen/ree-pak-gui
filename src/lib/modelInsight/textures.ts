import {
  modelInsightLoadTexturePreviews,
  type ModelInsightMeshAssets,
  type ModelTextureResolution
} from '@/api/tauri/utils'
import { logFrontendWarn } from '@/utils/frontendLog'
import { toUint8Array } from './bytes'
import type { PreviewModel } from './wasm'

export type ModelTextureUrls = Record<string, string>
export type ModelTextureImages = Record<string, HTMLImageElement>
export type ModelTextureLoadOptions = {
  textureResolution?: ModelTextureResolution
  warnScope?: string
  warnBasePath?: string
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

  const previews = await loadTexturePreviewsWithWarning(
    {
      belongsTo,
      baseEntryPath,
      texturePaths,
      textureResolution: options.textureResolution
    },
    options
  )
  const previewTexturePaths = new Set(previews.map((preview) => preview.texturePath))
  for (const texturePath of texturePaths) {
    if (!previewTexturePaths.has(texturePath)) {
      logModelTextureWarn(
        options,
        `texture preview missing base=${baseEntryPath} texture=${texturePath}`
      )
    }
  }

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
  textureUrls: ModelTextureUrls,
  options: ModelTextureLoadOptions = {}
): Promise<ModelTextureImages> {
  const entries = await Promise.all(
    Object.entries(textureUrls).map(async ([texturePath, url]) => {
      try {
        return [texturePath, await loadImage(url)] as const
      } catch (error) {
        logModelTextureWarn(
          options,
          `texture image failed texture=${texturePath} error=${error instanceof Error ? error.message : String(error)}`
        )
        return null
      } finally {
        URL.revokeObjectURL(url)
      }
    })
  )

  return Object.fromEntries(entries.filter((entry) => entry !== null))
}

async function loadTexturePreviewsWithWarning(
  options: Parameters<typeof modelInsightLoadTexturePreviews>[0],
  loadOptions: ModelTextureLoadOptions
) {
  try {
    return await modelInsightLoadTexturePreviews(options)
  } catch (error) {
    logModelTextureWarn(
      loadOptions,
      `texture preview request failed base=${options.baseEntryPath} textures=${options.texturePaths.length} error=${error instanceof Error ? error.message : String(error)}`
    )
    throw error
  }
}

function logModelTextureWarn(options: ModelTextureLoadOptions, message: string) {
  const context = options.warnBasePath ? `entry=${options.warnBasePath} ${message}` : message
  logFrontendWarn(options.warnScope ?? 'modelInsight.textures', context)
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
