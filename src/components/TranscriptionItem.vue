<script setup lang="ts">
import { ref, computed } from "vue";
import type { Transcription } from "../stores/pendingDelete";

const props = defineProps<{
  transcription: Transcription;
}>();

const emit = defineEmits<{
  (e: "delete", id: number): void;
  (e: "copy", text: string): void;
}>();

const showCopied = ref(false);

// Format relative time (e.g., "2 min ago")
const relativeTime = computed(() => {
  const now = new Date();
  const created = new Date(props.transcription.created_at);
  const diffMs = now.getTime() - created.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) return "Just now";
  if (diffMin < 60) return `${diffMin} min ago`;
  if (diffHour < 24) return `${diffHour} hour${diffHour > 1 ? "s" : ""} ago`;
  if (diffDay < 7) return `${diffDay} day${diffDay > 1 ? "s" : ""} ago`;

  return created.toLocaleDateString();
});

// Format language display
const languageDisplay = computed(() => {
  return props.transcription.language === "en" ? "English" : "German";
});

// Format duration (e.g., "5.2s")
const durationDisplay = computed(() => {
  const seconds = props.transcription.duration_ms / 1000;
  return `${seconds.toFixed(1)}s`;
});

async function handleCopy() {
  emit("copy", props.transcription.text);
  showCopied.value = true;
  setTimeout(() => {
    showCopied.value = false;
  }, 2000);
}

function handleDelete() {
  emit("delete", props.transcription.id);
}
</script>

<template>
  <div class="transcription-item card">
    <div class="item-content">
      <p class="item-text">{{ transcription.text }}</p>
      <div class="item-meta">
        <span>{{ relativeTime }}</span>
        <span class="meta-separator">-</span>
        <span>{{ languageDisplay }}</span>
        <span class="meta-separator">-</span>
        <span>{{ durationDisplay }}</span>
        <span class="meta-separator">-</span>
        <span>{{ transcription.word_count }} words</span>
      </div>
    </div>
    <div class="item-actions">
      <button
        class="btn btn-icon"
        :class="{ 'btn-success': showCopied }"
        @click="handleCopy"
        :aria-label="showCopied ? 'Copied!' : 'Copy to clipboard'"
        :title="showCopied ? 'Copied!' : 'Copy'"
      >
        <svg
          v-if="!showCopied"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
        <svg
          v-else
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
      </button>
      <button
        class="btn btn-icon btn-danger"
        @click="handleDelete"
        aria-label="Delete"
        title="Delete"
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="3 6 5 6 21 6"></polyline>
          <path
            d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
          ></path>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.transcription-item {
  display: flex;
  gap: 12px;
  margin-bottom: 8px;
  transition: box-shadow var(--transition-normal), transform var(--transition-fast);
  animation: scaleIn var(--transition-normal);
}

.transcription-item:hover {
  box-shadow: var(--shadow-md);
  transform: translateY(-1px);
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-text {
  color: var(--text-primary);
  line-height: 1.5;
  margin-bottom: 6px;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

.item-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  font-size: 12px;
  color: var(--text-secondary);
}

.meta-separator {
  color: var(--text-tertiary);
}

.item-actions {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
  opacity: 0.6;
  transition: opacity var(--transition-fast);
}

.transcription-item:hover .item-actions {
  opacity: 1;
}

.btn-success {
  color: var(--accent-success) !important;
  border-color: var(--accent-success) !important;
  animation: pulse 0.3s ease;
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.1);
  }
}
</style>
