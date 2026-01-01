<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import TranscriptionItem from "../components/TranscriptionItem.vue";
import {
  usePendingDelete,
  type Transcription,
} from "../stores/pendingDelete";

const transcriptions = ref<Transcription[]>([]);
const isLoading = ref(true);
const error = ref<string | null>(null);

// Use shared pending delete store (persists across navigation)
const {
  pendingDelete,
  scheduleDelete,
  undoDelete,
  registerCallbacks,
  unregisterCallbacks,
} = usePendingDelete();

// Event listener cleanup
let unlistenTranscriptionAdded: UnlistenFn | null = null;

const hasTranscriptions = computed(() => transcriptions.value.length > 0);

async function fetchHistory() {
  try {
    isLoading.value = true;
    error.value = null;
    const history = await invoke<Transcription[]>("get_history");
    transcriptions.value = history;
  } catch (e) {
    console.error("Failed to fetch history:", e);
    error.value = `Failed to load history: ${e}`;
  } finally {
    isLoading.value = false;
  }
}

async function handleCopy(text: string) {
  try {
    await writeText(text);
  } catch (e) {
    console.error("Failed to copy to clipboard:", e);
  }
}

function handleDelete(id: number) {
  // Find the transcription to delete
  const index = transcriptions.value.findIndex((t) => t.id === id);
  if (index === -1) return;

  const transcription = transcriptions.value[index];

  // Remove from list immediately
  transcriptions.value.splice(index, 1);

  // Schedule deletion with undo capability (handles previous pending delete internally)
  scheduleDelete(transcription);
}

function handleUndo() {
  const restored = undoDelete();
  if (!restored) return;

  // Restore the transcription to the list (at the correct position by created_at)
  restoreTranscription(restored);
}

function restoreTranscription(transcription: Transcription) {
  // Use created_at as primary sort key, id as secondary for deterministic ordering
  const insertIndex = transcriptions.value.findIndex((t) => {
    const tCreated = new Date(t.created_at).getTime();
    const restoreCreated = new Date(transcription.created_at).getTime();
    // If timestamps differ, sort by timestamp (newer first)
    if (tCreated !== restoreCreated) {
      return tCreated < restoreCreated;
    }
    // If timestamps are identical, use id as tiebreaker (higher id = newer)
    return t.id < transcription.id;
  });

  if (insertIndex === -1) {
    transcriptions.value.push(transcription);
  } else {
    transcriptions.value.splice(insertIndex, 0, transcription);
  }
}

// Unique key for this component's callback registration
const CALLBACK_KEY = "history-view";

onMounted(async () => {
  await fetchHistory();

  // Register callback for restoring items when undo is triggered from elsewhere
  registerCallbacks(CALLBACK_KEY, restoreTranscription);

  // Listen for new transcriptions
  unlistenTranscriptionAdded = await listen<Transcription>(
    "transcription-added",
    (event) => {
      // Prepend new transcription to the list
      transcriptions.value.unshift(event.payload);
    }
  );
});

onUnmounted(() => {
  // Cleanup event listener
  if (unlistenTranscriptionAdded) {
    unlistenTranscriptionAdded();
  }

  // Unregister callbacks but DON'T delete pending items - they persist and can be undone
  // when the user navigates back to the history view
  unregisterCallbacks(CALLBACK_KEY);
});
</script>

<template>
  <div class="history-view">
    <!-- Loading state -->
    <div v-if="isLoading" class="loading-state">
      <div class="loading-spinner"></div>
      <p>Loading history...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="error-state">
      <p>{{ error }}</p>
      <button class="btn btn-primary" @click="fetchHistory">Retry</button>
    </div>

    <!-- Empty state -->
    <div v-else-if="!hasTranscriptions" class="empty-state">
      <div class="empty-icon">
        <svg
          width="48"
          height="48"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"></path>
          <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
          <line x1="12" x2="12" y1="19" y2="22"></line>
        </svg>
      </div>
      <h2 class="empty-title">No transcriptions yet</h2>
      <p class="empty-description">
        Press your hotkey to start recording. Your transcriptions will appear
        here.
      </p>
    </div>

    <!-- Transcription list -->
    <div v-else class="transcription-list">
      <TranscriptionItem
        v-for="transcription in transcriptions"
        :key="transcription.id"
        :transcription="transcription"
        @copy="handleCopy"
        @delete="handleDelete"
      />
    </div>

    <!-- Undo toast -->
    <Transition name="toast">
      <div v-if="pendingDelete" class="undo-toast">
        <span>Transcription deleted</span>
        <button class="btn-undo" @click="handleUndo">Undo</button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.history-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
}

.transcription-list {
  flex: 1;
  overflow-y: auto;
}

/* Loading state */
.loading-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--text-secondary);
  animation: fadeIn var(--transition-normal);
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border-color);
  border-top-color: var(--accent-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Error state */
.error-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--accent-danger);
  text-align: center;
  padding: 32px;
}

/* Empty state */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 32px;
  color: var(--text-secondary);
  animation: fadeIn var(--transition-normal);
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.empty-description {
  font-size: 0.9rem;
  max-width: 280px;
  line-height: 1.5;
}

/* Undo toast */
.undo-toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-color-strong);
  border-radius: var(--radius-cards);
  box-shadow: var(--shadow-lg);
  z-index: 100;
}

.btn-undo {
  padding: 4px 12px;
  font-size: 13px;
  font-weight: 500;
  background: transparent;
  border: 1px solid var(--border-color-strong);
  border-radius: var(--radius-controls);
  color: var(--accent-primary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-undo:hover {
  background: var(--bg-hover);
  border-color: var(--accent-primary);
}

.btn-undo:active {
  background: var(--bg-active);
  transform: scale(0.98);
}

/* Toast animation */
.toast-enter-active,
.toast-leave-active {
  transition: all var(--transition-normal);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(20px);
}
</style>
