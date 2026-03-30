export type ExplorerSelectableItem = {
  id: string
}

export function toggleCheckedKey(checkedKeys: string[], key: string) {
  return checkedKeys.includes(key)
    ? checkedKeys.filter((checkedKey) => checkedKey !== key)
    : [...checkedKeys, key]
}

export function replaceCheckedKeysWithSingle(key: string) {
  return [key]
}

export function clearCheckedKeys() {
  return [] as string[]
}

export function appendRangeToCheckedKeys<TItem extends ExplorerSelectableItem>(
  checkedKeys: string[],
  items: TItem[],
  anchorKey: string,
  targetKey: string
) {
  const anchorIndex = items.findIndex((item) => item.id === anchorKey)
  const targetIndex = items.findIndex((item) => item.id === targetKey)

  if (anchorIndex === -1 || targetIndex === -1) {
    return checkedKeys.includes(targetKey) ? checkedKeys : [...checkedKeys, targetKey]
  }

  const [start, end] = anchorIndex <= targetIndex
    ? [anchorIndex, targetIndex]
    : [targetIndex, anchorIndex]
  const nextCheckedKeys = new Set(checkedKeys)

  for (let index = start; index <= end; index += 1) {
    const item = items[index]
    if (!item) continue
    nextCheckedKeys.add(item.id)
  }

  return [...nextCheckedKeys]
}

export function getOrderedCheckedItems<TItem extends ExplorerSelectableItem>(
  items: TItem[],
  checkedKeys: Iterable<string>
) {
  const checkedKeySet = new Set(checkedKeys)
  return items.filter((item) => checkedKeySet.has(item.id))
}
