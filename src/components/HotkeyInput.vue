<script setup lang="ts">
import { ref, watch, nextTick, useTemplateRef } from "vue";

defineProps<{
  modelValue: string;
  label: string;
  placeholder?: string;
  optional?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "recording-start"): void;
  (e: "recording-end"): void;
}>();

const isRecording = ref(false);
const inputRef = useTemplateRef<HTMLInputElement>("inputRef");

watch(isRecording, async (recording) => {
  if (recording) {
    await nextTick();
    inputRef.value?.focus();
  }
});

function toggleRecording() {
  const wasRecording = isRecording.value;
  isRecording.value = !isRecording.value;

  if (!wasRecording && isRecording.value) {
    emit("recording-start");
  } else if (wasRecording && !isRecording.value) {
    emit("recording-end");
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (!isRecording.value) return;
  e.preventDefault();

  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("CommandOrControl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");

  let key = e.key;
  if (key === " ") key = "Space";
  if (key.length === 1) key = key.toUpperCase();
  if (["Control", "Alt", "Shift", "Meta"].includes(key)) return;

  parts.push(key);
  emit("update:modelValue", parts.join("+"));
  isRecording.value = false;
  emit("recording-end");
}

// Allow parent to stop recording (when another input starts)
function stopRecording() {
  if (isRecording.value) {
    isRecording.value = false;
  }
}

defineExpose({ stopRecording, isRecording });
</script>

<template>
  <div class="field">
    <label :for="label" class="field-label">
      {{ label }}
      <span v-if="optional" class="optional">(optional)</span>
    </label>
    <div class="field-row">
      <input
        ref="inputRef"
        :id="label"
        type="text"
        class="input hotkey-input"
        :class="{ recording: isRecording }"
        :value="isRecording ? 'Press a key...' : (modelValue || '')"
        :placeholder="placeholder"
        readonly
        @keydown="handleKeydown"
      />
      <button class="btn" @click="toggleRecording">
        {{ isRecording ? "Cancel" : "Set" }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.field {
  margin-bottom: 12px;
}

.field-label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.optional {
  font-weight: 400;
  color: var(--text-muted);
}

.field-row {
  display: flex;
  gap: 8px;
}

.field-row .input {
  flex: 1;
}

.hotkey-input {
  font-family: var(--font-mono);
  font-size: 13px;
}

.hotkey-input.recording {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-muted);
}
</style>
