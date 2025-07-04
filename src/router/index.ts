import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/unpack'
    },
    {
      path: "/unpack",
      name: "UnpackView",
      component: () => import("@/pages/UnpackView.vue")
    },
    {
      path: "/pack",
      name: "PackView", 
      component: () => import("@/pages/PackView.vue")
    }
  ]
})

export default router
