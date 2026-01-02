<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { computed } from "vue";

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
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="m15 18-6-6 6-6"/>
        </svg>
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
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
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
