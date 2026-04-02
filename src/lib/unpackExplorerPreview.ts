import { resolveExplorerFileTypeKey } from '@/lib/explorerTypeTheme'
import type { ExplorerEntry, ExplorerLayoutMode } from '@/lib/unpackExplorer'

type PreviewableExplorerEntry = Pick<ExplorerEntry, 'children' | 'isDir' | 'name'>

export function isTextureExplorerEntry(item: Pick<ExplorerEntry, 'isDir' | 'name'>) {
  return !item.isDir && resolveExplorerFileTypeKey(item.name, item.isDir) === 'texture'
}

export function canOpenExplorerItemPreview(item: Pick<ExplorerEntry, 'isDir' | 'name'>) {
  return isTextureExplorerEntry(item)
}

export function getDefaultExplorerLayoutMode(
  directory: PreviewableExplorerEntry | null,
  savedLayout: ExplorerLayoutMode | null | undefined,
  texturePreviewEnabled: boolean
): ExplorerLayoutMode {
  if (texturePreviewEnabled && directory?.children.some((child) => isTextureExplorerEntry(child))) {
    return 'tile'
  }

  return savedLayout ?? 'details'
}
