import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface Transcription {
  id: number;
  text: string;
  language: string;
  duration_ms: number;
  word_count: number;
  created_at: string;
}

export interface PendingDelete {
  transcription: Transcription;
  timeoutId: number;
}

// Global state that persists across component navigation
const pendingDelete = ref<PendingDelete | null>(null);
const UNDO_TIMEOUT_MS = 5000;

// Callback for UI restore updates
let onRestoreCallback: ((transcription: Transcription) => void) | null = null;

async function permanentlyDelete(id: number) {
  try {
    await invoke<boolean>("delete_transcription", { id });
  } catch (e) {
    console.error("Failed to delete transcription:", e);
  }
}

export function usePendingDelete() {
  function registerCallbacks(
    onRestore: (transcription: Transcription) => void
  ) {
    onRestoreCallback = onRestore;
  }

  function unregisterCallbacks() {
    onRestoreCallback = null;
  }

  function scheduleDelete(transcription: Transcription) {
    // Clear any existing pending delete - fire and forget (no await to avoid UI blocking)
    if (pendingDelete.value) {
      clearTimeout(pendingDelete.value.timeoutId);
      // Don't await - let it run in background to keep UI responsive
      permanentlyDelete(pendingDelete.value.transcription.id);
    }

    // Set up undo timer
    const timeoutId = window.setTimeout(() => {
      // Timer expired, permanently delete
      permanentlyDelete(transcription.id);
      pendingDelete.value = null;
    }, UNDO_TIMEOUT_MS);

    pendingDelete.value = { transcription, timeoutId };
  }

  function undoDelete(): Transcription | null {
    if (!pendingDelete.value) return null;

    // Clear the timeout
    clearTimeout(pendingDelete.value.timeoutId);

    const restored = pendingDelete.value.transcription;
    pendingDelete.value = null;

    // Notify the UI if callback is registered
    if (onRestoreCallback) {
      onRestoreCallback(restored);
    }

    return restored;
  }

  function hasPendingDelete(): boolean {
    return pendingDelete.value !== null;
  }

  function getPendingDelete(): PendingDelete | null {
    return pendingDelete.value;
  }

  return {
    pendingDelete,
    scheduleDelete,
    undoDelete,
    hasPendingDelete,
    getPendingDelete,
    registerCallbacks,
    unregisterCallbacks,
  };
}
