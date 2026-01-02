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

const relativeTime = computed(() => {
  const now = new Date();
  const created = new Date(props.transcription.created_at);
  const diffMs = now.getTime() - created.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) return "Just now";
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHour < 24) return `${diffHour}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;

  return created.toLocaleDateString();
});

const languageLabel = computed(() => {
  const map: Record<string, string> = { en: "EN", de: "DE" };
  return map[props.transcription.language] ?? props.transcription.language.toUpperCase();
});

const duration = computed(() => {
  return `${(props.transcription.duration_ms / 1000).toFixed(1)}s`;
});

function handleCopy() {
  emit("copy", props.transcription.text);
  showCopied.value = true;
  setTimeout(() => { showCopied.value = false; }, 1500);
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
        :title="showCopied ? 'Copied!' : 'Copy'"
      >
        <svg v-if="!showCopied" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="9" y="9" width="13" height="13" rx="2"/>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
        </svg>
        <svg v-else width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"/>
        </svg>
      </button>
      <button class="action-btn action-btn-danger" @click="handleDelete" title="Delete">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 6h18"/>
          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
        </svg>
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
