import { describe, expect, test } from 'bun:test'
import { getSelectedItemRelativeRoot, normalizeDisplayPath } from './path'

describe('getSelectedItemRelativeRoot', () => {
  test('returns the parent path for a selected file', () => {
    expect(
      getSelectedItemRelativeRoot(
        'natives/STM/streaming/MasterMaterial/Textures/maka_flow_MSK3.tex.241106027'
      )
    ).toBe('natives/STM/streaming/MasterMaterial/Textures')
  })

  test('returns the parent path for a selected directory', () => {
    expect(getSelectedItemRelativeRoot('natives/STM/streaming/MasterMaterial/Textures')).toBe(
      'natives/STM/streaming/MasterMaterial'
    )
  })

  test('supports windows separators', () => {
    expect(
      getSelectedItemRelativeRoot(
        'natives\\STM\\streaming\\MasterMaterial\\Textures\\maka_flow_MSK3.tex.241106027'
      )
    ).toBe('natives/STM/streaming/MasterMaterial/Textures')
  })

  test('returns empty string when there is no parent path', () => {
    expect(getSelectedItemRelativeRoot('maka_flow_MSK3.tex.241106027')).toBe('')
  })
})

describe('normalizeDisplayPath', () => {
  test('normalizes separators without rewriting display spacing', () => {
    expect(normalizeDisplayPath(' A \\ B / C ')).toBe('A / B / C')
  })
})
