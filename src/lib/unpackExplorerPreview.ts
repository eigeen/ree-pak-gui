import { resolveExplorerFileTypeKey } from '@/lib/explorerTypeTheme'
import type { ExplorerEntry, ExplorerLayoutMode } from '@/lib/unpackExplorer'

type PreviewableExplorerEntry = Pick<ExplorerEntry, 'children' | 'isDir' | 'name'>

export type ExplorerPreviewKind = 'texture' | 'audioBank' | 'model'

export function getExplorerPreviewKind(
  item: Pick<ExplorerEntry, 'isDir' | 'name'>
): ExplorerPreviewKind | null {
  if (item.isDir) return null
  switch (resolveExplorerFileTypeKey(item.name, item.isDir)) {
    case 'texture':
      return 'texture'
    case 'sound':
      return 'audioBank'
    case 'mesh':
      return 'model'
    default:
      return null
  }
}

export function isTextureExplorerEntry(item: Pick<ExplorerEntry, 'isDir' | 'name'>) {
  return getExplorerPreviewKind(item) === 'texture'
}

export function isAudioBankExplorerEntry(item: Pick<ExplorerEntry, 'isDir' | 'name'>) {
  return getExplorerPreviewKind(item) === 'audioBank'
}

export function isModelExplorerEntry(item: Pick<ExplorerEntry, 'isDir' | 'name'>) {
  return getExplorerPreviewKind(item) === 'model'
}

export function canOpenExplorerItemPreview(item: Pick<ExplorerEntry, 'isDir' | 'name'>) {
  return getExplorerPreviewKind(item) !== null
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
