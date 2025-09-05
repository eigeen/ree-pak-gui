import { createRouter, createWebHistory } from 'vue-router'
import { getToolById } from '@/config/tools'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/unpack'
    },
    {
      path: '/unpack',
      name: 'UnpackView',
      component: () => import('@/pages/UnpackView.vue')
    },
    {
      path: '/pack',
      name: 'PackView',
      component: () => import('@/pages/PackView.vue')
    },
    {
      path: '/tools/:toolId',
      name: 'ToolsView',
      component: () => import('@/pages/ToolsView.vue'),
      beforeEnter: (to, from, next) => {
        const toolId = to.params.toolId as string
        const tool = getToolById(toolId)
        if (!tool) {
          // redirect to home if tool not found
          next('/')
        } else {
          next()
        }
      }
    }
  ]
})

export default router
