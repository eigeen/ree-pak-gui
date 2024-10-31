<script setup lang="ts">
import type { ExtractFileInfo, JsSafeHash, RenderTreeNode } from '@/api/tauri/pak'
import type { ElTree } from 'element-plus';
import { ref, watch } from 'vue';

// export interface TreeData {
//   label: string
//   hash?: JsSafeHash
//   isDir: boolean
//   children: TreeData[]
// }

export interface TreeData {
  // 唯一ID
  id: string
  // 显示名称
  label: string
  // 子节点
  children: TreeData[]
  // 属于哪个包
  belongingTo: string | undefined
}

export interface Props {
  data: RenderTreeNode | null;
  filterText?: string;
}

const props = defineProps<Props>()
// const props = withDefaults(defineProps<Props>(), {
//   data: (): RenderTreeNode[] => { return [] },
// })
const treeComponent = ref<InstanceType<typeof ElTree>>()
// 是否正在加载
const loading = ref(false)
// 缓存的已转换的 TreeData
const cachedTreeData = ref<TreeData[]>([])
// 已过滤的数据
const filteredData = ref<TreeData[]>([])

// 监听过滤器文本，应用过滤器
watch(() => props.filterText,
  (filterText) => {
    const filter = filterText ? filterText : ''
    console.log('applying filter', filter)

    filteredData.value = filterTreeData(deepCopy(cachedTreeData.value), filter)
  }
)

// 监听输入数据，输入变化时重新生成树
watch(() => props.data,
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
  let size;
  if (node.uncompressedSize !== undefined) {
    size = node.uncompressedSize
  } else {
    size = 0
  }

  let id;
  if (node.hash) {
    id = node.hash.toString()
  } else {
    id = `${node.name}_${Math.round(Math.random() * 10000000)}`
  }

  let data: TreeData = {
    id: id,
    label: `${node.name} (${formatSize(size)})`,
    children: node.children?.map(child => createTreeData(child)), // 递归处理子节点
    belongingTo: node.belongingTo
  }

  return data
}

function formatSize(size: number): string {
  if (size < 0) {
    return "Invalid";
  }

  const units = ["B", "KB", "MB", "GB", "TB"];
  let index = 0;

  while (size >= 1024 && index < units.length - 1) {
    size /= 1024;
    index++;
  }

  return `${size.toFixed(2)} ${units[index]}`;
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
  const lowerCaseText = text.toLowerCase();

  return data.map(node => {
    // 过滤子节点
    const filteredChildren = filterTreeData(node.children, text);

    // 判断当前节点是否包含关键词
    const isMatch = node.label.toLowerCase().includes(lowerCaseText);

    // 如果当前节点匹配或有匹配的子节点，则保留该节点
    if (isMatch || filteredChildren.length > 0) {
      return {
        ...node,
        children: filteredChildren // 只保留匹配的子节点
      };
    }

    // 如果当前节点和子节点都不匹配，则返回 null
    return null;
  }).filter(node => node !== null) as TreeData[];
}


function getCheckedNodes(): ExtractFileInfo[] {
  const nodes = treeComponent.value?.getCheckedNodes(true).filter(node => !node.isDir)
  if (!nodes) {
    return []
  }

  let result: ExtractFileInfo[] = []
  for (const node of nodes) {
    result.push({
      hash: parseId(node.id),
      belongsTo: node.belongingTo,
    })
  }

  console.log('result', result);

  return result
}

function parseId(id: string): JsSafeHash {
  return id.split(',').map(str => parseInt(str, 10)) as JsSafeHash
}

const treeProps = {
  value: 'id',
  label: 'label',
  children: 'children',
}

defineExpose({ getCheckedNodes })

</script>

<template>
  <el-tree-v2 ref="treeComponent" class="tree" :height="500" :props="treeProps" :data="filteredData" v-loading="loading"
    show-checkbox />
  <!-- <el-button @click="test">test</el-button> -->
</template>

<style scoped>
.tree {
  max-width: 800px;
  min-height: 500px;
  border: 1px solid var(--el-border-color);
  border-radius: 2px;
}
</style>