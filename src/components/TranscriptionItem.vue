<script setup lang="ts">
import { ref, computed, onUnmounted } from "vue";
import Icon from "./Icon.vue";
import type { Transcription } from "../stores/pendingDelete";
import { formatRelativeTime } from "../utils/time";
import { getLanguageLabel } from "../constants";

const props = defineProps<{
  transcription: Transcription;
}>();

const emit = defineEmits<{
  (e: "delete", id: number): void;
  (e: "copy", text: string): void;
}>();

const showCopied = ref(false);
let copyTimeoutId: ReturnType<typeof setTimeout> | null = null;

onUnmounted(() => {
  if (copyTimeoutId) {
    clearTimeout(copyTimeoutId);
  }
});

// Note: relativeTime uses new Date() which is not reactive. The time display
// won't automatically update (e.g., "Just now" to "1m ago"). This is a common
// and acceptable limitation for list items - the value updates on next render.
const relativeTime = computed(() => 
  formatRelativeTime(props.transcription.created_at)
);

const languageLabel = computed(() => 
  getLanguageLabel(props.transcription.language)
);

const duration = computed(() => {
  return `${(props.transcription.duration_ms / 1000).toFixed(1)}s`;
});

function handleCopy() {
  emit("copy", props.transcription.text);
  showCopied.value = true;
  if (copyTimeoutId) {
    clearTimeout(copyTimeoutId);
  }
  copyTimeoutId = setTimeout(() => { showCopied.value = false; }, 1500);
}

function handleDelete() {
  emit("delete", props.transcription.id);
}
</script>

<template>
  <article class="item card">
    <div class="content">
      <p class="text">{{ transcription.text }}</p>
      <div class="meta">
        <span class="tag" :class="'tag-' + transcription.language">{{ languageLabel }}</span>
        <span class="dot"></span>
        <span>{{ relativeTime }}</span>
        <span class="dot"></span>
        <span>{{ duration }}</span>
        <span class="dot"></span>
        <span>{{ transcription.word_count }} words</span>
      </div>
    </div>
    <div class="actions">
      <button
        class="action-btn"
        :class="{ copied: showCopied }"
        @click="handleCopy"
        :aria-label="showCopied ? 'Copied!' : 'Copy'"
        :title="showCopied ? 'Copied!' : 'Copy'"
      >
        <Icon v-if="!showCopied" name="copy" />
        <Icon v-else name="check" />
      </button>
      <button 
        class="action-btn action-btn-danger" 
        @click="handleDelete" 
        aria-label="Delete transcription"
        title="Delete"
      >
        <Icon name="trash" />
      </button>
    </div>
  </article>
</template>

<style scoped>
.item {
  display: flex;
  gap: 12px;
}

.content {
  flex: 1;
  min-width: 0;
}

.text {
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-primary);
  margin: 0 0 8px 0;
  word-wrap: break-word;
}

.meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-secondary);
}

.tag {
  padding: 3px 8px;
  border-radius: var(--radius-sm);
  font-weight: 500;
}

.tag-en {
  background: var(--accent-muted);
  color: var(--accent);
}

.tag-de {
  background: rgba(168, 85, 247, 0.15);
  color: #a855f7;
}

.dot {
  width: 3px;
  height: 3px;
  background: var(--text-muted);
  border-radius: 50%;
}

.actions {
  display: flex;
  flex-direction: column;
  gap: 4px;
  opacity: 0;
  transition: opacity var(--duration-fast) var(--ease);
}

.item:hover .actions {
  opacity: 1;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  padding: 0;
  background: var(--bg-muted);
  border: 1px solid var(--border-light);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease);
}

.action-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-default);
  color: var(--text-primary);
}

.action-btn.copied {
  background: var(--success-muted);
  border-color: var(--success);
  color: var(--success);
}

.action-btn-danger:hover {
  background: var(--danger-muted);
  border-color: var(--danger);
  color: var(--danger);
}
</style>
