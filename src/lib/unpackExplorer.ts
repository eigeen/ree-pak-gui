import type { CSSProperties } from 'vue'
import type { TreeData } from '@/components/FileTree.vue'

export type ExplorerEntry = TreeData & {
  children: ExplorerEntry[]
}

export type ExplorerDirectoryCounts = {
  folders: number
  files: number
}

export type ExplorerLayoutMode = 'tile' | 'details'

export type ExplorerBreadcrumbSegment = {
  id: string
  label: string
}

type ExplorerTexturePreviewResolver = (item: ExplorerEntry) => string | null
type ExplorerStyleResolver = (item: ExplorerEntry) => CSSProperties
type ExplorerIconResolver = (item: ExplorerEntry) => unknown
type ExplorerLabelResolver = (item: ExplorerEntry) => string
type ExplorerDirectoryCountsResolver = (item: ExplorerEntry) => ExplorerDirectoryCounts

export type ExplorerRenderers = {
  getTexturePreview: ExplorerTexturePreviewResolver
  getPreviewSurfaceStyle: ExplorerStyleResolver
  getHeroIcon: ExplorerIconResolver
  getHeroIconStyle: ExplorerStyleResolver
  getAccentStyle: ExplorerStyleResolver
  getItemTypeLabel: ExplorerLabelResolver
  getDirectoryCounts: ExplorerDirectoryCountsResolver
  getDetailText: ExplorerLabelResolver
}

export type ExplorerColumnLabels = {
  name: string
  type: string
  size: string
  details: string
}
