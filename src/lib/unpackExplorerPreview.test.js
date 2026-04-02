import { describe, expect, test } from 'bun:test'
import {
  canOpenExplorerItemPreview,
  getDefaultExplorerLayoutMode,
  isTextureExplorerEntry
} from './unpackExplorerPreview'

describe('unpackExplorerPreview', () => {
  test('treats texture files as previewable even when tile previews are disabled elsewhere', () => {
    const textureEntry = { isDir: false, name: 'maka_flow_MSK3.tex.241106027' }

    expect(isTextureExplorerEntry(textureEntry)).toBe(true)
    expect(canOpenExplorerItemPreview(textureEntry)).toBe(true)
  })

  test('falls back to details layout when texture preview is disabled', () => {
    const textureDirectory = {
      children: [{ isDir: false, name: 'maka_flow_MSK3.tex.241106027' }]
    }

    expect(getDefaultExplorerLayoutMode(textureDirectory, 'details', false)).toBe('details')
  })

  test('keeps tile layout as the default when texture preview is enabled', () => {
    const textureDirectory = {
      children: [{ isDir: false, name: 'maka_flow_MSK3.tex.241106027' }]
    }

    expect(getDefaultExplorerLayoutMode(textureDirectory, 'details', true)).toBe('tile')
  })
})
