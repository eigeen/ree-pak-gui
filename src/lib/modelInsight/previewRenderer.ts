import type { ModelTextureImages } from './textures'
import type { PreviewModel } from './wasm'
import {
  createDefaultModelPreviewCamera,
  createModelPreviewRenderState,
  disposeModelPreviewRenderState,
  renderModelPreviewFrame,
  setModelPreviewRenderMeshes,
  type ModelPreviewCameraOptions
} from './webglRenderer'

export interface ModelPreviewRenderOptions extends ModelPreviewCameraOptions {
  width?: number
  height?: number
}

export function renderModelPreviewToDataUrl(
  model: PreviewModel,
  options: ModelPreviewRenderOptions = {},
  textureImages: ModelTextureImages = {}
) {
  const width = Math.max(1, Math.floor(options.width ?? 256))
  const height = Math.max(1, Math.floor(options.height ?? 256))
  const canvas = document.createElement('canvas')
  canvas.width = width
  canvas.height = height

  const gl = canvas.getContext('webgl', {
    antialias: true,
    alpha: true,
    preserveDrawingBuffer: true
  })
  if (!gl) {
    throw new Error('This WebView does not support WebGL.')
  }

  const state = createModelPreviewRenderState(gl)
  try {
    setModelPreviewRenderMeshes(state, model, textureImages)
    renderModelPreviewFrame(state, createDefaultModelPreviewCamera(model, options))
    return canvas.toDataURL('image/png')
  } finally {
    disposeModelPreviewRenderState(state)
  }
}
