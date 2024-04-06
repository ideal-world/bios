import { createRouter, createWebHistory } from 'vue-router'
import * as bios from "bios-enhance-wasm"
import HomeView from '../views/HomeView.vue'
import User from '../views/User.vue'
import UserHome from '../views/UserHome.vue'
import UserPost from '../views/UserPost.vue'
import UserProfile from '../views/UserProfile.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/about',
      name: 'about',
      component: () => import('../views/AboutView.vue')
    },
    { path: '/users/:username/posts/:postId', component: UserPost },
    {
      path: '/user/:id',
      component: User,
      children: [
        { path: '', component: UserHome },
        {
          path: 'profile',
          component: UserProfile,
        },
      ],
    }
  ]
})

router.beforeEach((to, from) => {
  if (to.matched.length === 0) {
    try {
      const fullPath = bios.decrypt(to.fullPath.substring(1))
      router.replace({ path: fullPath })
    } catch (ignore) { }
  }
  return true
})

export default router
