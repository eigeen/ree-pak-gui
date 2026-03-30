import type { Component } from 'vue'

export type ContextMenuActionItem = {
  type: 'action'
  key: string
  label: string
  icon?: Component
  shortcut?: string
  disabled?: boolean
  destructive?: boolean
  action: () => void | Promise<void>
}

export type ContextMenuSeparatorItem = {
  type: 'separator'
  key: string
}

export type ContextMenuSubmenuItem = {
  type: 'submenu'
  key: string
  label: string
  icon?: Component
  disabled?: boolean
  children: ContextMenuEntry[]
}

export type ContextMenuEntry =
  | ContextMenuActionItem
  | ContextMenuSeparatorItem
  | ContextMenuSubmenuItem

export function compactContextMenuEntries(entries: ContextMenuEntry[]): ContextMenuEntry[] {
  const compacted: ContextMenuEntry[] = []

  for (const entry of entries) {
    if (entry.type === 'separator') {
      const previous = compacted[compacted.length - 1]
      if (!previous || previous.type === 'separator') {
        continue
      }
    }

    if (entry.type === 'submenu') {
      const children = compactContextMenuEntries(entry.children)
      if (children.length === 0) {
        continue
      }

      compacted.push({
        ...entry,
        children
      })
      continue
    }

    compacted.push(entry)
  }

  while (compacted[compacted.length - 1]?.type === 'separator') {
    compacted.pop()
  }

  return compacted
}
