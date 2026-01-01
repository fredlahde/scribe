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
    <div class="header-left">
      <button
        v-if="isSettingsPage"
        class="back-button"
        @click="goBack"
        aria-label="Go back"
      >
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </button>
    </div>
    <h1 class="title">
      {{ isSettingsPage ? "Settings" : "Scribe" }}
    </h1>
    <div class="header-right">
      <button
        v-if="!isSettingsPage"
        class="settings-button"
        @click="goToSettings"
        aria-label="Settings"
      >
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="3"></circle>
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
          ></path>
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
  padding: 12px 16px;
  background: var(--bg-toolbar);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--border-color);
  -webkit-app-region: drag;
  position: sticky;
  top: 0;
  z-index: 10;
}

.header-left,
.header-right {
  width: 40px;
  display: flex;
  align-items: center;
}

.header-right {
  justify-content: flex-end;
}

.title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  text-align: center;
  letter-spacing: -0.01em;
}

.settings-button,
.back-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  border: none;
  background: transparent;
  border-radius: var(--radius-controls);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  -webkit-app-region: no-drag;
}

.settings-button:hover,
.back-button:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.settings-button:active,
.back-button:active {
  background: var(--bg-active);
  transform: scale(0.95);
}

.settings-button svg,
.back-button svg {
  transition: transform var(--transition-fast);
}

.back-button:hover svg {
  transform: translateX(-2px);
}

.settings-button:hover svg {
  transform: rotate(30deg);
}
</style>
