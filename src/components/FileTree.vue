<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import type { ElTree, TreeNode } from 'element-plus'
import type { ExtractFileInfo, JsSafeHash, RenderTreeNode } from '@/api/tauri/pak'
import { Braces, File, FileCode2, FileText, Image, Link2, Package, Sparkles } from 'lucide-vue-next'

export interface TreeData {
  id: string
  label: string
  children: TreeData[]
  belongsTo: string | undefined
}

export interface Props {
  data: RenderTreeNode | null
  filterText?: string
  regexMode?: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'node-click', data: TreeData, node: TreeNode, event: MouseEvent): void
}>()

const treeComponent = ref<InstanceType<typeof ElTree>>()
const containerRef = ref<HTMLElement>()
const treeHeight = ref(200)
const cachedTreeData = ref<TreeData[]>([])
const filteredData = ref<TreeData[]>([])

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (!containerRef.value) return

  resizeObserver = new ResizeObserver((entries) => {
    let parentHeight = 0
    for (const entry of entries) {
      parentHeight = entry.contentRect.height
    }

    treeHeight.value = Math.max(200, parentHeight - 8)
  })

  resizeObserver.observe(containerRef.value)
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})

watch(
  () => [props.filterText, props.regexMode],
  () => doFilterTree()
)

watch(
  () => props.data,
  (data) => {
    if (!data) {
      cachedTreeData.value = []
      filteredData.value = []
      return
    }

    const treeData = createTreeData(data)
    cachedTreeData.value = [treeData]
    filteredData.value = props.filterText
      ? filterTreeData(deepCopy([treeData]), getFilterObject())
      : [treeData]
  },
  { immediate: true }
)

function getFilterObject() {
  const filter = props.filterText ?? ''
  return props.regexMode ? new RegExp(filter, 'i') : filter.toLowerCase()
}

function doFilterTree() {
  filteredData.value = filterTreeData(deepCopy(cachedTreeData.value), getFilterObject())
}

function createTreeData(node: RenderTreeNode): TreeData {
  const size =
    node.uncompressedSize !== undefined
      ? node.isCompressed
        ? node.uncompressedSize
        : node.compressedSize
      : 0

  return {
    id: node.hash ? node.hash.toString() : `${node.name}_${Math.round(Math.random() * 10000000)}`,
    label: `${node.name} (${formatSize(size)})`,
    children: node.children?.map((child) => createTreeData(child)) ?? [],
    belongsTo: node.belongsTo
  }
}

function formatSize(size: number): string {
  if (size < 0) return 'Invalid'

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let index = 0

  while (size >= 1024 && index < units.length - 1) {
    size /= 1024
    index++
  }

  return `${size.toFixed(2)} ${units[index]}`
}

function deepCopy(data: TreeData[]): TreeData[] {
  return JSON.parse(JSON.stringify(data))
}

function filterTreeData(data: TreeData[], filter: string | RegExp): TreeData[] {
  return data
    .map((node) => {
      const filteredChildren = filterTreeData(node.children, filter)

      let isMatch = false
      if (typeof filter === 'string') {
        isMatch = filter === '' || node.label.toLowerCase().includes(filter)
      } else {
        isMatch = filter.test(node.label)
      }

      if (isMatch || filteredChildren.length > 0) {
        return {
          ...node,
          children: filteredChildren
        }
      }

      return null
    })
    .filter((node): node is TreeData => node !== null)
}

function getCheckedNodes(): ExtractFileInfo[] {
  const nodes = treeComponent.value?.getCheckedNodes(true).filter((node) => !node.isDir)
  if (!nodes) return []

  return nodes.map((node) => ({
    hash: parseId(node.id),
    belongsTo: node.belongsTo
  }))
}

function parseId(id: string): JsSafeHash {
  return id.split(',').map((str) => parseInt(str, 10)) as JsSafeHash
}

const extIconMap = {
  msg: FileText,
  user: Braces,
  tex: Image,
  chain: Link2,
  chain2: Link2,
  pfb: Package,
  efx: Sparkles,
  mesh: FileCode2
}

function getFileIcon(label: string) {
  const path = label.split('(').at(0)?.trim() ?? ''
  const pathComponents = path.split('.')
  if (pathComponents.length < 3) {
    return File
  }

  const ext = pathComponents.at(-2)?.toLowerCase()
  return (ext && extIconMap[ext as keyof typeof extIconMap]) || File
}

const treeProps = {
  value: 'id',
  label: 'label',
  children: 'children'
}

defineExpose({ getCheckedNodes })
</script>

<template>
  <div ref="containerRef" class="h-full">
    <el-tree-v2
      ref="treeComponent"
      :data="filteredData"
      :height="treeHeight"
      :props="treeProps"
      class="rounded-[1.15rem] bg-transparent"
      show-checkbox
      @node-click="
        (data: TreeData, node: TreeNode, e: MouseEvent) => {
          emit('node-click', data, node, e)
        }
      "
    >
      <template #default="{ node }">
        <div class="flex items-center gap-2 text-sm">
          <component :is="getFileIcon(node.label)" v-if="node.isLeaf" class="size-4 text-primary" />
          <span class="truncate">{{ node.label }}</span>
        </div>
      </template>
    </el-tree-v2>
  </div>
</template>
