import {
  Archive,
  Box,
  File,
  FileCode2,
  FileImage,
  FileMusic,
  Folder,
  Layers3,
  Sparkles
} from 'lucide-vue-next'
import { getReAssetExtension } from '@/lib/reAssetPath'

type ExplorerThemeDefinition = {
  key: string
  label: string
  accent: string
  hero: string
  surfaceGlow: string
}

type ExplorerFileTypeDefinition = {
  key: string
  label: string
  icon: unknown
  extensions: string[]
  themeKey: ExplorerThemeKey
}

export const explorerThemePresets = [
  {
    key: 'amber',
    label: 'Amber',
    accent: '#e2b15f',
    hero: '#f3d594',
    surfaceGlow: 'rgba(255,218,150,0.18)'
  },
  {
    key: 'blue',
    label: 'Blue',
    accent: '#7aa2ff',
    hero: '#b9cbff',
    surfaceGlow: 'rgba(138,165,255,0.16)'
  },
  {
    key: 'violet',
    label: 'Violet',
    accent: '#8d63ff',
    hero: '#ccb7ff',
    surfaceGlow: 'rgba(141,99,255,0.18)'
  },
  {
    key: 'emerald',
    label: 'Emerald',
    accent: '#5ecf93',
    hero: '#a8ecc4',
    surfaceGlow: 'rgba(94,207,147,0.18)'
  },
  {
    key: 'rose',
    label: 'Rose',
    accent: '#ff8ca1',
    hero: '#ffc0cb',
    surfaceGlow: 'rgba(255,140,161,0.18)'
  },
  {
    key: 'orange',
    label: 'Orange',
    accent: '#ffad66',
    hero: '#ffd3a9',
    surfaceGlow: 'rgba(255,173,102,0.18)'
  },
  {
    key: 'cyan',
    label: 'Cyan',
    accent: '#5fd4e2',
    hero: '#a8edf3',
    surfaceGlow: 'rgba(95,212,226,0.18)'
  },
  {
    key: 'slate',
    label: 'Slate',
    accent: '#8b96ac',
    hero: '#d7dceb',
    surfaceGlow: 'rgba(139,150,172,0.16)'
  }
] as const satisfies readonly ExplorerThemeDefinition[]

export type ExplorerThemeKey = (typeof explorerThemePresets)[number]['key']

export const explorerFileTypes = [
  {
    key: 'folder',
    label: 'Folder',
    icon: Folder,
    extensions: [],
    themeKey: 'amber'
  },
  {
    key: 'texture',
    label: 'Texture',
    icon: FileImage,
    extensions: ['tex'],
    themeKey: 'violet'
  },
  {
    key: 'material',
    label: 'Material',
    icon: Layers3,
    extensions: ['mdf2', 'mdf', 'mtl'],
    themeKey: 'emerald'
  },
  {
    key: 'mesh',
    label: 'Mesh',
    icon: Box,
    extensions: ['mesh'],
    themeKey: 'cyan'
  },
  {
    key: 'motlist',
    label: 'MotList',
    icon: Sparkles,
    extensions: ['mot', 'motlist'],
    themeKey: 'rose'
  },
  {
    key: 'sound',
    label: 'Sound',
    icon: FileMusic,
    extensions: ['bnk', 'sbnk', 'pck', 'spck'],
    themeKey: 'orange'
  },
  {
    key: 'prefab',
    label: 'Prefab',
    icon: Box,
    extensions: ['pfb'],
    themeKey: 'slate'
  },
  {
    key: 'asset',
    label: 'Asset',
    icon: File,
    extensions: [],
    themeKey: 'blue'
  }
] as const satisfies readonly ExplorerFileTypeDefinition[]

export type ExplorerFileTypeKey = (typeof explorerFileTypes)[number]['key']

const fileTypeByKey = new Map(explorerFileTypes.map((item) => [item.key, item]))
const themeByKey = new Map(explorerThemePresets.map((item) => [item.key, item]))
const extensionToTypeKey = new Map<string, ExplorerFileTypeKey>()
const fallbackFileTypeDefinition: ExplorerFileTypeDefinition = {
  key: 'asset',
  label: 'Asset',
  icon: File,
  extensions: [],
  themeKey: 'blue'
}
const fallbackThemeDefinition: ExplorerThemeDefinition = {
  key: 'blue',
  label: 'Blue',
  accent: '#7aa2ff',
  hero: '#b9cbff',
  surfaceGlow: 'rgba(138,165,255,0.16)'
}

for (const type of explorerFileTypes) {
  for (const extension of type.extensions) {
    extensionToTypeKey.set(extension, type.key)
  }
}

const knownExplorerExtensions = new Set(extensionToTypeKey.keys())

export function getExplorerFileExtension(pathOrName: string): string {
  return getReAssetExtension(pathOrName, knownExplorerExtensions)
}

export function resolveExplorerFileTypeKey(name: string, isDir: boolean): ExplorerFileTypeKey {
  if (isDir) {
    return 'folder'
  }

  const extension = getExplorerFileExtension(name)
  return extensionToTypeKey.get(extension) ?? 'asset'
}

export function getExplorerFileTypeDefinition(
  typeKey: ExplorerFileTypeKey
): ExplorerFileTypeDefinition {
  return fileTypeByKey.get(typeKey) ?? fallbackFileTypeDefinition
}

export function getExplorerThemeDefinition(themeKey: ExplorerThemeKey): ExplorerThemeDefinition {
  return themeByKey.get(themeKey) ?? fallbackThemeDefinition
}

export function getExplorerThemeKeyForType(typeKey: ExplorerFileTypeKey): ExplorerThemeKey {
  return getExplorerFileTypeDefinition(typeKey).themeKey
}

export function getExplorerThemeForType(typeKey: ExplorerFileTypeKey): ExplorerThemeDefinition {
  return getExplorerThemeDefinition(getExplorerThemeKeyForType(typeKey))
}
