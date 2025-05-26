<script setup lang="ts">
import type { ExtractFileInfo, JsSafeHash, RenderTreeNode } from '@/api/tauri/pak'
import type { ElTree } from 'element-plus'
import { ref, watch, onMounted, onUnmounted } from 'vue'

export interface TreeData {
  // 唯一ID
  id: string
  // 显示名称
  label: string
  // 子节点
  children: TreeData[]
  // 属于哪个包
  belongsTo: string | undefined
}

export interface Props {
  data: RenderTreeNode | null
  filterText?: string
}

const props = defineProps<Props>()
// const props = withDefaults(defineProps<Props>(), {
//   data: (): RenderTreeNode[] => { return [] },
// })
const treeComponent = ref<InstanceType<typeof ElTree>>()
const containerRef = ref<HTMLElement>()
const treeHeight = ref(200)
// 是否正在加载
const loading = ref(false)
// 缓存的已转换的 TreeData
const cachedTreeData = ref<TreeData[]>([])
// 已过滤的数据
const filteredData = ref<TreeData[]>([])
let resizeObserver: ResizeObserver | null = null
let lazyUpdateTimeout: number | undefined

// 监听容器大小变化
onMounted(() => {
  if (containerRef.value) {
    resizeObserver = new ResizeObserver((entries) => {
      let parentHeight = 0
      for (const entry of entries) {
        const { height } = entry.contentRect
        parentHeight = height
      }

      // 减去边框高度2和一个预留空间5
      // 不留额外空间会导致缩小时外层容器不变化，导致无法缩小
      treeHeight.value = Math.max(200, parentHeight - 7)
    })
    resizeObserver.observe(containerRef.value)
  }
})

// 组件卸载时清理监听
onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
  }
  clearTimeout(lazyUpdateTimeout)
})

// 监听过滤器文本，应用过滤器
watch(
  () => props.filterText,
  (filterText) => {
    const filter = filterText ? filterText : ''
    console.log('applying filter', filter)

    filteredData.value = filterTreeData(deepCopy(cachedTreeData.value), filter)
  }
)

// 监听输入数据，输入变化时重新生成树
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
    filteredData.value = cachedTreeData.value
  }
)

// 从输入的树格式 RenderTreeNode 转换成显示用的 TreeData 格式
function createTreeData(node: RenderTreeNode): TreeData {
  let size: number = 0
  if (node.uncompressedSize !== undefined) {
    size = node.isCompressed ? node.uncompressedSize : node.compressedSize
  } else {
    size = 0
  }

  let id
  if (node.hash) {
    id = node.hash.toString()
  } else {
    id = `${node.name}_${Math.round(Math.random() * 10000000)}`
  }

  let data: TreeData = {
    id: id,
    label: `${node.name} (${formatSize(size)})`,
    children: node.children?.map((child) => createTreeData(child)), // 递归处理子节点
    belongsTo: node.belongsTo
  }

  return data
}

function formatSize(size: number): string {
  if (size < 0) {
    return 'Invalid'
  }

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

// // 关键词过滤
// const filterTreeData = (data: TreeData[], text: string): TreeData[] => {
//   if (!text) {
//     return data
//   }

//   return data.map((node) => {
//     const filteredChildren = filterTreeData(node.children, text);
//     // 保留非空目录节点，包含过滤文本的叶子节点
//     const filter = (node.isDir && filteredChildren.length > 0) || (!node.isDir && node.label.includes(text));

//     if (filter) {
//       return {
//         ...node,
//         children: filteredChildren,
//       };
//     }
//     return null;
//   }).filter((node): node is TreeData => node !== null);
// }

// 通过关键词过滤树的叶子结点，返回新的树
function filterTreeData(data: TreeData[], text: string): TreeData[] {
  const lowerCaseText = text.toLowerCase()

  return data
    .map((node) => {
      // 过滤子节点
      const filteredChildren = filterTreeData(node.children, text)

      // 判断当前节点是否包含关键词
      const isMatch = node.label.toLowerCase().includes(lowerCaseText)

      // 如果当前节点匹配或有匹配的子节点，则保留该节点
      if (isMatch || filteredChildren.length > 0) {
        return {
          ...node,
          children: filteredChildren // 只保留匹配的子节点
        }
      }

      // 如果当前节点和子节点都不匹配，则返回 null
      return null
    })
    .filter((node) => node !== null) as TreeData[]
}

function getCheckedNodes(): ExtractFileInfo[] {
  const nodes = treeComponent.value?.getCheckedNodes(true).filter((node) => !node.isDir)
  if (!nodes) {
    return []
  }

  let result: ExtractFileInfo[] = []
  for (const node of nodes) {
    result.push({
      hash: parseId(node.id),
      belongsTo: node.belongsTo
    })
  }

  console.log('result', result)

  return result
}

function parseId(id: string): JsSafeHash {
  return id.split(',').map((str) => parseInt(str, 10)) as JsSafeHash
}

const extIconMap: Record<string, string> = {
  msg: 'mdi-text-box',
  user: 'mdi-code-braces',
  tex: 'mdi-image',
  chain: 'mdi-link-variant',
  chain2: 'mdi-link-variant',
  pfb: 'mdi-package-variant',
  efx: 'mdi-auto-fix',
  mesh: 'mdi-vector-polygon'
}

function getFileIcon(label: string): string {
  const path = label.split('(')[0].trim()
  const pathComponents = path.split('.')
  if (pathComponents.length < 3) {
    return 'mdi-file'
  }
  const ext = pathComponents[pathComponents.length - 2].toLowerCase()
  if (extIconMap[ext]) {
    return extIconMap[ext]
  }
  return 'mdi-file'
}

const treeProps = {
  value: 'id',
  label: 'label',
  children: 'children'
}

defineExpose({ getCheckedNodes })
</script>

<template>
  <div class="tree-container" ref="containerRef">
    <el-tree-v2
      ref="treeComponent"
      class="tree"
      :props="treeProps"
      :data="filteredData"
      :height="treeHeight"
      v-loading="loading"
      show-checkbox
    >
      <template #default="{ node }">
        <v-icon
          v-if="node.isLeaf"
          :icon="getFileIcon(node.label)"
          class="prefix"
          size="small"
        ></v-icon>
        <span>{{ node.label }}</span>
      </template>
    </el-tree-v2>
  </div>
</template>

<style scoped>
.tree-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.tree {
  flex: 1;
  min-height: 0;
}

.prefix {
  color: rgb(51, 133, 255);
  margin-right: 4px;
}
</style>
