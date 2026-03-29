import {
  Archive,
  Box,
  File,
  FileCode2,
  Folder,
  Image,
  Layers3,
  Music4,
  Sparkles
} from 'lucide-vue-next'

export type ExplorerThemeKey =
  | 'amber'
  | 'blue'
  | 'violet'
  | 'emerald'
  | 'rose'
  | 'orange'
  | 'cyan'
  | 'slate'

export type ExplorerFileTypeKey =
  | 'folder'
  | 'asset'
  | 'texture'
  | 'material'
  | 'mesh'
  | 'motlist'
  | 'sound'

type ExplorerThemeDefinition = {
  key: ExplorerThemeKey
  label: string
  accent: string
  hero: string
  surfaceGlow: string
}

type ExplorerFileTypeDefinition = {
  key: ExplorerFileTypeKey
  label: string
  icon: unknown
  extensions: string[]
  defaultTheme: ExplorerThemeKey
  configurable: boolean
}

export const explorerThemePresets: ExplorerThemeDefinition[] = [
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
]

export const explorerFileTypes: ExplorerFileTypeDefinition[] = [
  {
    key: 'folder',
    label: 'Folder',
    icon: Folder,
    extensions: [],
    defaultTheme: 'amber',
    configurable: false
  },
  {
    key: 'texture',
    label: 'Texture',
    icon: Image,
    extensions: ['tex'],
    defaultTheme: 'violet',
    configurable: true
  },
  {
    key: 'material',
    label: 'Material',
    icon: Layers3,
    extensions: ['mdf2', 'mdf', 'mtl'],
    defaultTheme: 'emerald',
    configurable: true
  },
  {
    key: 'mesh',
    label: 'Mesh',
    icon: Box,
    extensions: ['mesh'],
    defaultTheme: 'cyan',
    configurable: true
  },
  {
    key: 'motlist',
    label: 'MotList',
    icon: Sparkles,
    extensions: ['mot', 'motlist'],
    defaultTheme: 'rose',
    configurable: true
  },
  {
    key: 'sound',
    label: 'Sound',
    icon: Music4,
    extensions: ['bnk', 'sbnk', 'pck', 'spck'],
    defaultTheme: 'orange',
    configurable: true
  },
  {
    key: 'asset',
    label: 'Asset',
    icon: File,
    extensions: [],
    defaultTheme: 'blue',
    configurable: true
  }
]

const fileTypeByKey = new Map(explorerFileTypes.map((item) => [item.key, item]))
const themeByKey = new Map(explorerThemePresets.map((item) => [item.key, item]))
const extensionToTypeKey = new Map<string, ExplorerFileTypeKey>()
const fallbackFileTypeDefinition: ExplorerFileTypeDefinition = {
  key: 'asset',
  label: 'Asset',
  icon: File,
  extensions: [],
  defaultTheme: 'blue',
  configurable: true
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

export const defaultExplorerTypeThemes = explorerFileTypes.reduce(
  (result, type) => {
    result[type.key] = type.defaultTheme
    return result
  },
  {} as Record<ExplorerFileTypeKey, ExplorerThemeKey>
)

export const configurableExplorerFileTypes = explorerFileTypes.filter((item) => item.configurable)

export function getExplorerFileExtension(name: string): string {
  if (typeof name !== 'string') {
    return ''
  }

  const matched = /\.([^.]+)$/.exec(name.toLowerCase())
  return matched?.[1] ?? ''
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

export function getExplorerResolvedThemeKey(
  typeKey: ExplorerFileTypeKey,
  overrides?: Partial<Record<ExplorerFileTypeKey, ExplorerThemeKey>>
): ExplorerThemeKey {
  return overrides?.[typeKey] ?? defaultExplorerTypeThemes[typeKey]
}

export function getExplorerResolvedTheme(
  typeKey: ExplorerFileTypeKey,
  overrides?: Partial<Record<ExplorerFileTypeKey, ExplorerThemeKey>>
): ExplorerThemeDefinition {
  return getExplorerThemeDefinition(getExplorerResolvedThemeKey(typeKey, overrides))
}
