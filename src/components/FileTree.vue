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
const filteredData = ref<TreeData[]>([])
const loading = ref(false)

// watch(() => props.filterText,
//   (filterText) => {
//     const filter = filterText ? filterText : ''
//     console.log('applying filter', filter)

//     filteredData.value = sortAndMerge(filterTreeData(deepCopy(props.data), filter))
//   }
// )
watch(() => props.data,
  (data) => {

    if (!data) {
      filteredData.value = []
      return
    }

    const treeData = createTreeData(data)
    filteredData.value = [treeData]
  }
)

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
        return "Invalid size";
    }
    
    const units = ["B", "KB", "MB", "GB", "TB"];
    let index = 0;

    while (size >= 1024 && index < units.length - 1) {
        size /= 1024;
        index++;
    }

    return `${size.toFixed(2)} ${units[index]}`;
}

// const deepCopy = (data: TreeData[]): TreeData[] => {
//   // return structuredClone(data)
//   return JSON.parse(JSON.stringify(data))
// }

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