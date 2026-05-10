import type { PakInfo, RenderTreeNode } from '@/api/tauri/pak'
import type { ExplorerLayoutMode } from '@/lib/unpackExplorer'
import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'

export type UnpackSidebarTab = 'resources' | 'tree'

function buildPakSignature(paks: Pick<PakInfo, 'id' | 'path'>[]): string {
  return paks.map((pak) => `${pak.id}:${pak.path}`).join('\n')
}

export const useUnpackStore = defineStore('unpack-page', () => {
  const pakData = ref<PakInfo[]>([])
  const treeData = shallowRef<RenderTreeNode[] | null>(null)
  const showOverlay = ref(false)
  const loadingTree = ref(false)
  const currentDirectoryKey = ref('')
  const treeFocusKey = ref('')
  const checkedTreeKeys = ref<string[]>([])
  const focusedEntryKey = ref('')
  const checkedEntryKeys = ref<string[]>([])
  const selectionAnchorKey = ref('')
  const explorerSearchText = ref('')
  const filterDraftText = ref('')
  const filterTextApply = ref('')
  const sidebarTab = ref<UnpackSidebarTab>('resources')
  const explorerLayoutMode = ref<ExplorerLayoutMode>('details')
  const expandedTreeKeys = ref<string[]>([])
  const loadedTreeSourceKey = ref('')

  function setPakData(nextPakData: PakInfo[]): boolean {
    const changed = buildPakSignature(pakData.value) !== buildPakSignature(nextPakData)
    pakData.value = nextPakData
    if (changed) resetTreeState()
    return changed
  }

  function setRenderedTree(
    nextTreeData: RenderTreeNode[],
    sourceKey: string,
    expandedKeys: string[]
  ) {
    treeData.value = nextTreeData
    loadedTreeSourceKey.value = sourceKey
    expandedTreeKeys.value = expandedKeys
    sidebarTab.value = 'tree'
    showOverlay.value = false
  }

  function resetTreeState() {
    treeData.value = null
    loadedTreeSourceKey.value = ''
    expandedTreeKeys.value = []
    resetNavigationState()
  }

  function resetNavigationState() {
    currentDirectoryKey.value = ''
    treeFocusKey.value = ''
    checkedTreeKeys.value = []
    clearExplorerSelection()
  }

  function clearExplorerSelection() {
    focusedEntryKey.value = ''
    checkedEntryKeys.value = []
    selectionAnchorKey.value = ''
  }

  return {
    pakData,
    treeData,
    showOverlay,
    loadingTree,
    currentDirectoryKey,
    treeFocusKey,
    checkedTreeKeys,
    focusedEntryKey,
    checkedEntryKeys,
    selectionAnchorKey,
    explorerSearchText,
    filterDraftText,
    filterTextApply,
    sidebarTab,
    explorerLayoutMode,
    expandedTreeKeys,
    loadedTreeSourceKey,
    setPakData,
    setRenderedTree,
    resetTreeState,
    resetNavigationState,
    clearExplorerSelection
  }
})
