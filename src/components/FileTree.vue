<script setup lang="ts">
import type { TreeNode, TreeNodeData } from 'element-plus'
import type { TreeV2Instance } from 'element-plus/es/components/tree-v2/src/instance'
import type { TreeData } from '@/lib/unpackTree'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { Folder } from 'lucide-vue-next'
import { ElTreeV2 } from 'element-plus'

interface Props {
  data: TreeData[] | null
  currentNodeKey?: string
  checkedKeys?: string[]
  expandedKeys?: string[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'node-click', data: TreeData, node: TreeNode, event: MouseEvent): void
  (e: 'node-contextmenu', data: TreeData, node: TreeNode, event: MouseEvent): void
  (e: 'node-check', checkedKeys: string[]): void
  (e: 'node-expanded-change', expandedKeys: string[]): void
  (e: 'background-contextmenu', event: MouseEvent): void
}>()

const treeComponent = ref<TreeV2Instance>()
const containerRef = ref<HTMLElement>()
const treeHeight = ref(200)
const treeData = computed(() => props.data ?? [])
const expandedKeySet = ref(new Set(props.expandedKeys ?? []))

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
  () => props.currentNodeKey,
  (key) => {
    if (!key) return
    treeComponent.value?.setCurrentKey?.(key)
  }
)

watch(
  () => props.checkedKeys ?? [],
  (keys) => {
    const checkedKeys = ((treeComponent.value as any)?.getCheckedKeys?.(false) ?? []).map(String)
    if (hasSameKeys(checkedKeys, keys)) return
    ;(treeComponent.value as any)?.setCheckedKeys?.(keys)
  },
  { deep: true }
)

watch(
  () => props.expandedKeys ?? [],
  (keys) => {
    if (hasSameKeys([...expandedKeySet.value], keys)) return
    expandedKeySet.value = new Set(keys)
    treeComponent.value?.setExpandedKeys(keys)
  },
  { deep: true }
)

const treeProps = {
  value: 'id',
  label: 'label',
  children: 'children'
}

type TreeNodeWithParent = TreeNode & {
  parent?: TreeNodeWithParent
  key: string | number
}

const treeClass = computed(() => ['desktop-tree editor-scrollbar rounded-[0.7rem] bg-transparent'])

function bringNodeIntoView(key: string) {
  if (!key) return

  const node = treeComponent.value?.getNode(key)
  if (!node) return

  const expandedKeys = mergeExpandedKeys(getAncestorKeys(node))

  treeComponent.value?.setExpandedKeys(expandedKeys)
  treeComponent.value?.setCurrentKey?.(key)
  treeComponent.value?.scrollToNode(key, 'center')
  setExpandedKeys(expandedKeys)
}

function collapseAll() {
  treeComponent.value?.setExpandedKeys([])
  setExpandedKeys([])
}

function getAncestorKeys(node: TreeNode) {
  const keys: string[] = []
  let cursor = (node as TreeNodeWithParent).parent

  while (cursor) {
    keys.unshift(String(cursor.key))
    cursor = cursor.parent
  }

  return keys
}

function mergeExpandedKeys(keys: string[]) {
  return [...new Set([...expandedKeySet.value, ...keys])]
}

function setExpandedKeys(keys: string[]) {
  if (hasSameKeys([...expandedKeySet.value], keys)) return
  expandedKeySet.value = new Set(keys)
  emit('node-expanded-change', keys)
}

function hasSameKeys(currentKeys: string[], nextKeys: string[]) {
  if (currentKeys.length !== nextKeys.length) return false

  const currentKeySet = new Set(currentKeys)
  return nextKeys.every((key) => currentKeySet.has(key))
}

function handleBackgroundContextMenu(event: MouseEvent) {
  const target = event.target
  if (target instanceof Element && target.closest('.el-tree-node')) {
    return
  }

  event.preventDefault()
  emit('background-contextmenu', event)
}

function toTreeData(data: TreeNodeData): TreeData {
  return data as TreeData
}

function handleNodeClick(data: TreeNodeData, node: TreeNode, event: MouseEvent) {
  emit('node-click', toTreeData(data), node, event)
}

function handleNodeContextMenu(event: Event, data: TreeNodeData, node: TreeNode) {
  event.preventDefault()
  emit('node-contextmenu', toTreeData(data), node, event as MouseEvent)
}

function handleNodeCheck() {
  const checkedKeys = ((treeComponent.value as any)?.getCheckedKeys?.(false) ?? []).map(String)
  emit('node-check', checkedKeys)
}

function handleNodeExpand(data: TreeNodeData) {
  setExpandedKeys(mergeExpandedKeys([toTreeData(data).id]))
}

function handleNodeCollapse(data: TreeNodeData) {
  const collapsedKey = toTreeData(data).id
  setExpandedKeys([...expandedKeySet.value].filter((key) => key !== collapsedKey))
}

defineExpose({ bringNodeIntoView, collapseAll })
</script>

<template>
  <div ref="containerRef" class="h-full" @contextmenu="handleBackgroundContextMenu">
    <el-tree-v2
      ref="treeComponent"
      :current-node-key="currentNodeKey"
      :default-checked-keys="props.checkedKeys ?? []"
      :data="treeData"
      :height="treeHeight"
      :props="treeProps"
      :class="treeClass"
      :default-expanded-keys="props.expandedKeys ?? []"
      highlight-current
      node-key="id"
      show-checkbox
      @node-click="handleNodeClick"
      @check="handleNodeCheck"
      @node-expand="handleNodeExpand"
      @node-collapse="handleNodeCollapse"
      @node-contextmenu="handleNodeContextMenu"
    >
      <template #default="{ node }">
        <div class="flex w-full min-w-0 items-center gap-2 text-sm">
          <Folder class="size-3.5 shrink-0 text-amber-200" />
          <span class="min-w-0 truncate">{{ node.data.label }}</span>
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
