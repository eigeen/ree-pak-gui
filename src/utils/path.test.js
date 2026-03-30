import { describe, expect, test } from 'bun:test'
import { getSelectedItemRelativeRoot } from './path'

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
