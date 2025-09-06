import type { Component } from 'vue'
import { defineAsyncComponent } from 'vue'

// 工具路由接口
export interface ToolRoute {
  id: string // 工具ID
  title: string // 工具标题(i18n key)
  icon?: string // 工具图标
  component: Component // 工具组件
}

// 工具配置注册表
const toolsRegistry: ToolRoute[] = [
  {
    id: 'path-scan',
    title: 'pathScanner.title',
    icon: 'mdi-folder-search',
    component: defineAsyncComponent(() => import('@/components/tools/PathScanner.vue'))
  }
]

// 获取所有工具
export function getAllTools(): ToolRoute[] {
  return [...toolsRegistry]
}

// 根据ID查找工具
export function getToolById(id: string): ToolRoute | undefined {
  return toolsRegistry.find((tool) => tool.id === id)
}
