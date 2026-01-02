import { createRouter, createWebHashHistory } from "vue-router";
import HistoryView from "./views/HistoryView.vue";
import SettingsView from "./views/SettingsView.vue";

const routes = [
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
