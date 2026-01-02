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
import { CALLBACK_KEYS } from "../constants";

const transcriptions = ref<Transcription[]>([]);
const isLoading = ref(true);
const error = ref<string | null>(null);
const copyError = ref<string | null>(null);

const {
  pendingDelete,
  scheduleDelete,
  undoDelete,
  registerCallbacks,
  unregisterCallbacks,
} = usePendingDelete();

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
    copyError.value = null;
  } catch (e) {
    console.error("Failed to copy to clipboard:", e);
    copyError.value = "Failed to copy to clipboard";
    setTimeout(() => {
      copyError.value = null;
    }, 3000);
  }
}

function handleDelete(id: number) {
  const index = transcriptions.value.findIndex((t) => t.id === id);
  if (index === -1) return;

  const transcription = transcriptions.value[index];
  transcriptions.value.splice(index, 1);
  scheduleDelete(transcription);
}

function handleUndo() {
  undoDelete();
}

/**
 * Restores a transcription to its correct position in the list.
 * List is sorted by created_at descending (newest first), with id as tiebreaker.
 */
function restoreTranscription(transcription: Transcription) {
  const insertIndex = transcriptions.value.findIndex((t) => {
    const tCreated = new Date(t.created_at).getTime();
    const restoreCreated = new Date(transcription.created_at).getTime();
    if (tCreated !== restoreCreated) {
      return tCreated < restoreCreated;
    }
    return t.id < transcription.id;
  });

  if (insertIndex === -1) {
    transcriptions.value.push(transcription);
  } else {
    transcriptions.value.splice(insertIndex, 0, transcription);
  }
}

onMounted(async () => {
  await fetchHistory();
  registerCallbacks(CALLBACK_KEYS.HISTORY_VIEW, restoreTranscription);

  unlistenTranscriptionAdded = await listen<Transcription>(
    "transcription-added",
    (event) => {
      transcriptions.value.unshift(event.payload);
    }
  );
});

onUnmounted(() => {
  if (unlistenTranscriptionAdded) {
    unlistenTranscriptionAdded();
  }
  unregisterCallbacks(CALLBACK_KEYS.HISTORY_VIEW);
});
</script>

<template>
  <div class="history-view">
    <!-- Loading -->
    <div v-if="isLoading" class="state-box">
      <div class="spinner"></div>
      <p class="state-text">Loading...</p>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="state-box">
      <div class="state-icon state-icon-error">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"/>
          <path d="m15 9-6 6"/>
          <path d="m9 9 6 6"/>
        </svg>
      </div>
      <p class="state-text">{{ error }}</p>
      <button class="btn btn-primary" @click="fetchHistory">Retry</button>
    </div>

    <!-- Empty -->
    <div v-else-if="!hasTranscriptions" class="state-box">
      <div class="state-icon">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"/>
          <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
          <line x1="12" x2="12" y1="19" y2="22"/>
        </svg>
      </div>
      <h3 class="state-title">No transcriptions yet</h3>
      <p class="state-text">Press your hotkey to start recording</p>
    </div>

    <!-- List -->
    <div v-else class="list">
      <TranscriptionItem
        v-for="item in transcriptions"
        :key="item.id"
        :transcription="item"
        @copy="handleCopy"
        @delete="handleDelete"
      />
    </div>

    <!-- Undo toast -->
    <Transition name="toast">
      <div v-if="pendingDelete" class="toast">
        <span>Deleted</span>
        <button class="toast-btn" @click="handleUndo">Undo</button>
      </div>
    </Transition>

    <!-- Copy error toast -->
    <Transition name="toast">
      <div v-if="copyError" class="toast toast-error">
        <span>{{ copyError }}</span>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.history-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

/* State boxes */
.state-box {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 40px 20px;
  gap: 12px;
}

.state-icon {
  color: var(--text-muted);
  margin-bottom: 4px;
}

.state-icon-error {
  color: var(--danger);
}

.state-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.state-text {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
  max-width: 240px;
}

/* Spinner */
.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border-light);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Toast */
.toast {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: var(--text-primary);
  color: var(--text-inverse);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  font-size: 13px;
  z-index: 100;
}

.toast-error {
  background: var(--danger);
  bottom: 70px;
}

.toast-btn {
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
  background: rgba(255, 255, 255, 0.1);
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease);
}

.toast-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.toast-enter-active,
.toast-leave-active {
  transition: all var(--duration-normal) var(--ease);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(12px);
}
</style>
