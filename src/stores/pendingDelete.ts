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
  timeoutId: ReturnType<typeof setTimeout>;
}

// Global state that persists across component navigation
const pendingDelete = ref<PendingDelete | null>(null);
const UNDO_TIMEOUT_MS = 5000;

// Registry of restore callbacks by component instance (using unique keys)
const restoreCallbacks = new Map<string, (transcription: Transcription) => void>();

// Queue for pending deletions to ensure they complete in order
let deletionInProgress: Promise<void> = Promise.resolve();

async function permanentlyDelete(id: number): Promise<void> {
  try {
    await invoke<boolean>("delete_transcription", { id });
  } catch (e) {
    console.error("Failed to delete transcription:", e);
    // Re-throw to allow callers to handle the error
    throw e;
  }
}

// Chain a deletion operation to ensure proper ordering
function queueDeletion(id: number): Promise<void> {
  const deletion = deletionInProgress.then(() => permanentlyDelete(id)).catch((e) => {
    // Log but don't break the chain
    console.error("Deletion failed for id:", id, e);
  });
  deletionInProgress = deletion;
  return deletion;
}

export function usePendingDelete() {
  function registerCallbacks(
    key: string,
    onRestore: (transcription: Transcription) => void
  ) {
    restoreCallbacks.set(key, onRestore);
  }

  function unregisterCallbacks(key: string) {
    restoreCallbacks.delete(key);
  }

  async function scheduleDelete(transcription: Transcription): Promise<void> {
    // If there's a pending delete, finalize it first (await to prevent data loss)
    if (pendingDelete.value) {
      clearTimeout(pendingDelete.value.timeoutId);
      const previousId = pendingDelete.value.transcription.id;
      pendingDelete.value = null;
      
      // Queue the previous deletion and wait for it
      await queueDeletion(previousId);
    }

    // Set up undo timer for the new deletion
    const timeoutId = setTimeout(() => {
      if (pendingDelete.value?.transcription.id === transcription.id) {
        // Timer expired, permanently delete
        queueDeletion(transcription.id).catch((e) => {
          console.error("Failed to delete after timeout:", e);
          // On failure, notify all registered callbacks to restore the item
          restoreCallbacks.forEach((callback) => {
            try {
              callback(transcription);
            } catch (callbackError) {
              console.error("Restore callback failed:", callbackError);
            }
          });
        });
        pendingDelete.value = null;
      }
    }, UNDO_TIMEOUT_MS);

    pendingDelete.value = { transcription, timeoutId };
  }

  function undoDelete(): Transcription | null {
    if (!pendingDelete.value) return null;

    // Clear the timeout
    clearTimeout(pendingDelete.value.timeoutId);

    const restored = pendingDelete.value.transcription;
    pendingDelete.value = null;

    // Notify all registered callbacks
    restoreCallbacks.forEach((callback) => {
      try {
        callback(restored);
      } catch (e) {
        console.error("Restore callback failed:", e);
      }
    });

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
