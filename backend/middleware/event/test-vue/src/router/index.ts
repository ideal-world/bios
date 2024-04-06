import { createRouter, createWebHistory } from "vue-router";
// import * as bios from "bios-enhance-wasm";
import EventRegister from "../views/EventRegister.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "eventRegister",
      component: EventRegister,
    },
  ],
});

router.beforeEach((to, from) => {
  if (to.matched.length === 0) {
    try {
      // const fullPath = bios.decrypt(to.fullPath.substring(1));
      router.replace({ path: to.fullPath.substring(1) });
    } catch (ignore) {}
  }
  return true;
});

export default router;
