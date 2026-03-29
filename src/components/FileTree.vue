<script setup lang="ts">
import type { TreeNode } from 'element-plus'
import type { TreeV2Instance } from 'element-plus/es/components/tree-v2/src/instance'
import type { ExtractFileInfo, JsSafeHash, RenderTreeNode } from '@/api/tauri/pak'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { Folder } from 'lucide-vue-next'

export interface TreeData {
  id: string
  name: string
  label: string
  path: string
  parentId?: string
  hash?: JsSafeHash
  isDir: boolean
  sizeText: string
  children: TreeData[]
  belongsTo: string | undefined
}

interface Props {
  data: RenderTreeNode[] | null
  filterText?: string
  regexMode?: boolean
  currentNodeKey?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'node-click', data: TreeData, node: TreeNode, event: MouseEvent): void
}>()

const treeComponent = ref<TreeV2Instance>()
const containerRef = ref<HTMLElement>()
const treeHeight = ref(200)
const cachedTreeData = ref<TreeData[]>([])
const cachedFullTreeData = ref<TreeData[]>([])
const filteredData = ref<TreeData[]>([])

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (!containerRef.value) return

  resizeObserver = new ResizeObserver((entries) => {
    const entry = entries[0]
    if (!entry) return
    treeHeight.value = Math.max(200, entry.contentRect.height - 8)
  })

  resizeObserver.observe(containerRef.value)
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})

watch(
  () => [props.filterText, props.regexMode],
  () => {
    filteredData.value = filterTreeData(deepCopy(cachedTreeData.value), getFilterObject())
  }
)

watch(
  () => props.currentNodeKey,
  (key) => {
    if (!key) return
    treeComponent.value?.setCurrentKey?.(key)
  }
)

watch(
  () => props.data,
  (data) => {
    if (!data) {
      cachedTreeData.value = []
      filteredData.value = []
      return
    }

    const nextTree = data.map((node) => createTreeData(node))
    cachedFullTreeData.value = nextTree
    cachedTreeData.value = nextTree
      .map((node) => pruneFiles(node))
      .filter((node): node is TreeData => Boolean(node))
    filteredData.value = props.filterText
      ? filterTreeData(deepCopy(cachedTreeData.value), getFilterObject())
      : deepCopy(cachedTreeData.value)
  },
  { immediate: true }
)

function getFilterObject() {
  const filter = props.filterText ?? ''
  return props.regexMode ? new RegExp(filter, 'i') : filter.toLowerCase()
}

function createTreeData(node: RenderTreeNode, parentPath = '', parentId?: string): TreeData {
  const id = node.hash ? node.hash.toString() : `${parentPath}/${node.name}`
  const path = parentPath ? `${parentPath}/${node.name}` : node.name

  return {
    id,
    name: node.name,
    label: node.name,
    path,
    parentId,
    hash: node.hash,
    isDir: node.isDir,
    sizeText: formatSize(
      node.uncompressedSize !== undefined
        ? node.isCompressed
          ? node.uncompressedSize
          : node.compressedSize
        : 0
    ),
    children: node.children?.map((child) => createTreeData(child, path, id)) ?? [],
    belongsTo: node.belongsTo
  }
}

function formatSize(size: number): string {
  if (size < 0) return 'Invalid'

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let index = 0
  let current = size

  while (current >= 1024 && index < units.length - 1) {
    current /= 1024
    index++
  }

  return `${current.toFixed(2)} ${units[index]}`
}

function deepCopy(data: TreeData[]): TreeData[] {
  return JSON.parse(JSON.stringify(data))
}

function pruneFiles(node: TreeData): TreeData | null {
  if (!node.isDir) {
    return null
  }

  return {
    ...node,
    children: node.children
      .map((child) => pruneFiles(child))
      .filter((child): child is TreeData => child !== null)
  }
}

function filterTreeData(data: TreeData[], filter: string | RegExp): TreeData[] {
  return data
    .map((node) => {
      const filteredChildren = filterTreeData(node.children, filter)
      const isMatch =
        typeof filter === 'string'
          ? filter === '' || node.path.toLowerCase().includes(filter)
          : filter.test(node.path)

      if (!isMatch && filteredChildren.length === 0) {
        return null
      }

      return {
        ...node,
        children: filteredChildren
      }
    })
    .filter((node): node is TreeData => node !== null)
}

function getCheckedNodes(): ExtractFileInfo[] {
  const checkedDirectories = treeComponent.value?.getCheckedNodes().filter((node) => node.isDir)
  if (!checkedDirectories?.length) return []

  const files = new Map<string, ExtractFileInfo>()

  const collectFiles = (node: TreeData) => {
    if (!node.isDir) {
      if (node.hash && node.belongsTo) {
        files.set(node.id, {
          hash: node.hash,
          belongsTo: node.belongsTo
        })
      }
      return
    }

    node.children.forEach(collectFiles)
  }

  const fullTreeMap = new Map<string, TreeData>()
  const walk = (node: TreeData) => {
    fullTreeMap.set(node.id, node)
    node.children.forEach(walk)
  }
  cachedFullTreeData.value.forEach(walk)

  checkedDirectories.forEach((node) => {
    const source = fullTreeMap.get(node.id)
    if (source) {
      collectFiles(source)
    }
  })

  return [...files.values()]
}

const treeProps = {
  value: 'id',
  label: 'label',
  children: 'children'
}

const treeClass = computed(() => ['desktop-tree editor-scrollbar rounded-[0.7rem] bg-transparent'])

function bringNodeIntoView(key: string) {
  if (!key) return

  const node = treeComponent.value?.getNode(key)
  if (!node) return

  const expandedKeys: string[] = []
  let cursor = node.parent

  while (cursor) {
    expandedKeys.unshift(String(cursor.key))
    cursor = cursor.parent
  }

  treeComponent.value?.setExpandedKeys(expandedKeys)
  treeComponent.value?.setCurrentKey?.(key)
  treeComponent.value?.scrollToNode(key, 'center')
}

function collapseAll() {
  treeComponent.value?.setExpandedKeys([])
}

defineExpose({ bringNodeIntoView, collapseAll, getCheckedNodes })
</script>

<template>
  <div ref="containerRef" class="h-full">
    <el-tree-v2
      ref="treeComponent"
      :current-node-key="currentNodeKey"
      :data="filteredData"
      :height="treeHeight"
      :props="treeProps"
      :class="treeClass"
      highlight-current
      node-key="id"
      show-checkbox
      @node-click="
        (data: TreeData, node: TreeNode, e: MouseEvent) => emit('node-click', data, node, e)
      "
    >
      <template #default="{ node }">
        <div class="flex w-full items-center gap-2 text-sm">
          <Folder class="size-3.5 shrink-0 text-amber-200" />
          <span class="truncate">{{ node.data.label }}</span>
        </div>
      </template>
    </el-tree-v2>
  </div>
</template>

<style scoped>
:deep(.el-tree) {
  background-color: transparent;
}

:deep(.el-tree-node.is-current > .el-tree-node__content) {
  color: var(--color-foreground);
}

:deep(.el-tree-node.is-current > .el-tree-node__content .text-primary),
:deep(.el-tree-node.is-current > .el-tree-node__content .text-muted-foreground),
:deep(.el-tree-node.is-current > .el-tree-node__content span) {
  color: var(--color-foreground);
}
</style>
