import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";
import HistoryView from "./views/HistoryView.vue";
import SettingsView from "./views/SettingsView.vue";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "history",
    component: HistoryView,
  },
  {
    path: "/settings",
    name: "settings",
    component: SettingsView,
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
