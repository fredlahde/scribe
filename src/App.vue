<script setup lang="ts">
import AppHeader from "./components/AppHeader.vue";
</script>

<template>
  <div class="app">
    <AppHeader />
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-base);
}

.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}
</style>
