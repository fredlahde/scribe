<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import Icon from "../components/Icon.vue";
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

const totalWords = computed(() => {
  return transcriptions.value.reduce(
    (sum, transcription) => sum + transcription.word_count,
    0
  );
});

const totalWordsLabel = computed(() => {
  return totalWords.value === 1 ? "word" : "words";
});

const averageWpm = computed(() => {
  let totalMinutes = 0;
  transcriptions.value.forEach((transcription) => {
    totalMinutes += transcription.duration_ms / 60000;
  });
  if (totalMinutes <= 0 || totalWords.value === 0) return 0;
  return Math.round(totalWords.value / totalMinutes);
});

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
    <div class="stats-panel">
      <div class="stat-item">
        <span class="stat-label">Average speed</span>
        <span class="stat-value">
          <span class="stat-number">{{ averageWpm }}</span>
          <span class="stat-unit">WPM</span>
        </span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Total words dictated</span>
        <span class="stat-value">
          <span class="stat-number">{{ totalWords.toLocaleString() }}</span>
          <span class="stat-unit">{{ totalWordsLabel }}</span>
        </span>
      </div>
    </div>

    <div class="history-body">
      <!-- Loading -->
      <div v-if="isLoading" class="state-box">
        <div class="spinner"></div>
        <p class="state-text">Loading...</p>
      </div>

      <!-- Error -->
      <div v-else-if="error" class="state-box">
        <div class="state-icon state-icon-error">
          <Icon name="error" :size="24" />
        </div>
        <p class="state-text">{{ error }}</p>
        <button class="btn btn-primary" @click="fetchHistory">Retry</button>
      </div>

      <!-- Empty -->
      <div v-else-if="!hasTranscriptions" class="state-box">
        <div class="state-icon">
          <Icon name="microphone" :size="32" />
        </div>
        <h3 class="state-title">No transcriptions yet</h3>
        <p class="state-text">Press your hotkey to start recording</p>
      </div>

      <!-- List -->
      <TransitionGroup v-else name="list" tag="div" class="list">
        <TranscriptionItem
          v-for="item in transcriptions"
          :key="item.id"
          :transcription="item"
          @copy="handleCopy"
          @delete="handleDelete"
        />
      </TransitionGroup>
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
  gap: 24px;
}

.stats-panel {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-light);
  background: linear-gradient(140deg, rgba(28, 28, 28, 0.98), rgba(10, 10, 10, 0.98));
  box-shadow: var(--shadow-sm);
  overflow: hidden;
  flex-shrink: 0;
}

.history-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding-bottom: 12px;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.history-body::-webkit-scrollbar {
  width: 0;
  height: 0;
}

.stat-item {
  --stat-accent: #38bdf8;
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 14px 18px 16px;
  min-height: 70px;
}

.stat-item:not(:first-child) {
  border-left: 1px solid var(--border-light);
}

.stat-item::after {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 2px;
  background: linear-gradient(90deg, var(--stat-accent), transparent 75%);
  opacity: 0.85;
}

.stat-item:nth-child(2) {
  --stat-accent: #f97316;
}

.stat-label {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: rgba(255, 255, 255, 0.9);
}

.stat-label::before {
  content: "";
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--stat-accent);
  box-shadow: 0 0 10px color-mix(in srgb, var(--stat-accent) 70%, transparent);
}

.stat-value {
  display: flex;
  align-items: baseline;
  gap: 6px;
  color: var(--text-primary);
  line-height: 1.1;
}

.stat-number {
  font-size: 22px;
  font-weight: 600;
  letter-spacing: -0.02em;
  font-variant-numeric: tabular-nums;
}

.stat-unit {
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: rgba(255, 255, 255, 0.7);
}

@media (max-width: 540px) {
  .stats-panel {
    grid-template-columns: 1fr;
  }

  .stat-item:not(:first-child) {
    border-left: none;
    border-top: 1px solid var(--border-light);
  }
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
}

/* List transitions */
.list-enter-active,
.list-leave-active {
  transition: opacity 0.3s var(--ease), transform 0.3s var(--ease);
}

.list-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

.list-move {
  transition: transform 0.3s var(--ease);
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
