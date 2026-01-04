<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { computed } from "vue";
import Icon from "./Icon.vue";

const route = useRoute();
const router = useRouter();

const isSettingsPage = computed(() => route.path === "/settings");

function goToSettings() {
  router.push("/settings");
}

function goBack() {
  router.push("/");
}
</script>

<template>
  <header class="header">
    <div class="header-side">
      <button
        v-if="isSettingsPage"
        class="icon-btn"
        @click="goBack"
        aria-label="Go back"
      >
        <Icon name="back" />
      </button>
    </div>
    
    <h1 class="title">{{ isSettingsPage ? "Settings" : "Scribe" }}</h1>
    
    <div class="header-side header-side-end">
      <button
        v-if="!isSettingsPage"
        class="icon-btn"
        @click="goToSettings"
        aria-label="Settings"
      >
        <Icon name="settings" />
      </button>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-light);
  -webkit-app-region: drag;
}

.header-side {
  width: 36px;
  display: flex;
  align-items: center;
}

.header-side-end {
  justify-content: flex-end;
}

.title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  letter-spacing: -0.01em;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease);
  -webkit-app-region: no-drag;
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.icon-btn:active {
  background: var(--bg-active);
}
</style>
