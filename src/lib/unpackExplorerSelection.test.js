import { describe, expect, test } from 'bun:test'
import {
  appendRangeToCheckedKeys,
  clearCheckedKeys,
  getOrderedCheckedItems,
  replaceCheckedKeysWithSingle,
  toggleCheckedKey
} from './unpackExplorerSelection'

const items = [{ id: 'a' }, { id: 'b' }, { id: 'c' }, { id: 'd' }]

describe('unpackExplorerSelection', () => {
  test('toggles a checked key on and off', () => {
    expect(toggleCheckedKey([], 'a')).toEqual(['a'])
    expect(toggleCheckedKey(['a', 'b'], 'a')).toEqual(['b'])
  })

  test('replaces the checked set with a single key', () => {
    expect(replaceCheckedKeysWithSingle('c')).toEqual(['c'])
  })

  test('clears checked keys', () => {
    expect(clearCheckedKeys()).toEqual([])
  })

  test('appends a visible range without removing checked keys outside the range', () => {
    expect(appendRangeToCheckedKeys(['d'], items, 'a', 'c')).toEqual(['d', 'a', 'b', 'c'])
  })

  test('falls back to checking the target when the anchor is unavailable', () => {
    expect(appendRangeToCheckedKeys([], items, 'missing', 'b')).toEqual(['b'])
  })

  test('returns checked items in view order', () => {
    expect(getOrderedCheckedItems(items, ['d', 'b'])).toEqual([{ id: 'b' }, { id: 'd' }])
  })
})
