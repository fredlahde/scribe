<script setup lang="ts">
import { ref, onMounted } from "vue";
import { load } from "@tauri-apps/plugin-store";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useRouter } from "vue-router";

const router = useRouter();

interface Settings {
  hotkey: string;
  hotkey_de: string;
  hotkey_mute: string;
  model_path: string | null;
}

const settings = ref<Settings>({
  hotkey: "F2",
  hotkey_de: "",
  hotkey_mute: "F4",
  model_path: null,
});

const isRecordingHotkey = ref(false);
const isRecordingHotkeyDe = ref(false);
const isRecordingHotkeyMute = ref(false);
const showModelWarning = ref(false);
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let store: any = null;

onMounted(async () => {
  store = await load("settings.json");

  const savedHotkey = await store.get("hotkey");
  const savedHotkeyDe = await store.get("hotkey_de");
  const savedHotkeyMute = await store.get("hotkey_mute");
  const savedModelPath = await store.get("model_path");

  if (savedHotkey) settings.value.hotkey = savedHotkey;
  if (savedHotkeyDe) settings.value.hotkey_de = savedHotkeyDe;
  if (savedHotkeyMute) settings.value.hotkey_mute = savedHotkeyMute;
  if (savedModelPath) settings.value.model_path = savedModelPath;

  showModelWarning.value = !settings.value.model_path;
});

async function browseModel() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Whisper Model",
          extensions: ["bin"],
        },
      ],
    });

    if (selected) {
      settings.value.model_path = selected as string;
      showModelWarning.value = false;
    }
  } catch (err) {
    console.error("Failed to open file dialog:", err);
  }
}

function startRecordingHotkey(type: "en" | "de" | "mute") {
  if (type === "en") {
    isRecordingHotkey.value = !isRecordingHotkey.value;
    isRecordingHotkeyDe.value = false;
    isRecordingHotkeyMute.value = false;
  } else if (type === "de") {
    isRecordingHotkeyDe.value = !isRecordingHotkeyDe.value;
    isRecordingHotkey.value = false;
    isRecordingHotkeyMute.value = false;
  } else {
    isRecordingHotkeyMute.value = !isRecordingHotkeyMute.value;
    isRecordingHotkey.value = false;
    isRecordingHotkeyDe.value = false;
  }
}

function handleKeydown(e: KeyboardEvent, type: "en" | "de" | "mute") {
  const isRecording =
    (type === "en" && isRecordingHotkey.value) ||
    (type === "de" && isRecordingHotkeyDe.value) ||
    (type === "mute" && isRecordingHotkeyMute.value);

  if (!isRecording) return;

  e.preventDefault();

  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("CommandOrControl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");

  let key = e.key;
  if (key === " ") key = "Space";
  if (key.length === 1) key = key.toUpperCase();

  // Skip modifier-only presses
  if (["Control", "Alt", "Shift", "Meta"].includes(key)) return;

  parts.push(key);
  const hotkeyStr = parts.join("+");

  if (type === "en") {
    settings.value.hotkey = hotkeyStr;
    isRecordingHotkey.value = false;
  } else if (type === "de") {
    settings.value.hotkey_de = hotkeyStr;
    isRecordingHotkeyDe.value = false;
  } else {
    settings.value.hotkey_mute = hotkeyStr;
    isRecordingHotkeyMute.value = false;
  }
}

async function saveSettings() {
  if (!store) return;

  await store.set("hotkey", settings.value.hotkey);
  await store.set("hotkey_de", settings.value.hotkey_de || "");
  await store.set("hotkey_mute", settings.value.hotkey_mute || "F4");
  await store.set("model_path", settings.value.model_path);
  await store.save();

  try {
    await invoke("reload_settings");
  } catch (e) {
    console.error("Failed to reload settings:", e);
  }

  router.push("/");
}

function cancel() {
  router.push("/");
}
</script>

<template>
  <div class="settings-view">
    <!-- Warning banner when no model configured -->
    <div v-if="showModelWarning" class="warning">
      Please select a Whisper model file to get started.
    </div>

    <!-- Model Path -->
    <div class="setting">
      <label class="setting-label">Whisper Model:</label>
      <div class="input-row">
        <input
          type="text"
          class="input"
          :value="settings.model_path || ''"
          readonly
          placeholder="No model selected"
        />
        <button class="btn" @click="browseModel">Browse...</button>
      </div>
      <small class="setting-hint">Select a .bin model file (e.g., ggml-medium.bin)</small>
    </div>

    <!-- English Hotkey -->
    <div class="setting">
      <label class="setting-label">Push-to-Talk Hotkey (English):</label>
      <div class="input-row">
        <input
          type="text"
          class="input"
          :value="isRecordingHotkey ? 'Press a key...' : settings.hotkey"
          readonly
          placeholder="Press to record..."
          @keydown="handleKeydown($event, 'en')"
        />
        <button class="btn" @click="startRecordingHotkey('en')">
          {{ isRecordingHotkey ? "Cancel" : "Record" }}
        </button>
      </div>
      <small class="setting-hint">Current: {{ settings.hotkey }}</small>
    </div>

    <!-- German Hotkey -->
    <div class="setting">
      <label class="setting-label">Push-to-Talk Hotkey (German):</label>
      <div class="input-row">
        <input
          type="text"
          class="input"
          :value="isRecordingHotkeyDe ? 'Press a key...' : (settings.hotkey_de || '')"
          readonly
          placeholder="Press to record..."
          @keydown="handleKeydown($event, 'de')"
        />
        <button class="btn" @click="startRecordingHotkey('de')">
          {{ isRecordingHotkeyDe ? "Cancel" : "Record" }}
        </button>
      </div>
      <small class="setting-hint">
        Current: {{ settings.hotkey_de || "Not set" }}
      </small>
    </div>

    <!-- Mute Hotkey -->
    <div class="setting">
      <label class="setting-label">Mute/Unmute Hotkey:</label>
      <div class="input-row">
        <input
          type="text"
          class="input"
          :value="isRecordingHotkeyMute ? 'Press a key...' : (settings.hotkey_mute || 'F4')"
          readonly
          placeholder="Press to record..."
          @keydown="handleKeydown($event, 'mute')"
        />
        <button class="btn" @click="startRecordingHotkey('mute')">
          {{ isRecordingHotkeyMute ? "Cancel" : "Record" }}
        </button>
      </div>
      <small class="setting-hint">
        Current: {{ settings.hotkey_mute || "F4" }}
      </small>
    </div>

    <!-- Buttons -->
    <div class="buttons">
      <button class="btn" @click="cancel">Cancel</button>
      <button class="btn btn-primary" @click="saveSettings">Save</button>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  max-width: 400px;
  margin: 0 auto;
  animation: fadeIn var(--transition-normal);
}

.setting {
  margin-bottom: 20px;
}

.setting-label {
  display: block;
  font-weight: 500;
  margin-bottom: 6px;
  color: var(--text-primary);
}

.setting-hint {
  display: block;
  color: var(--text-secondary);
  margin-top: 4px;
  font-size: 12px;
}

.input-row {
  display: flex;
  gap: 8px;
}

.input-row .input {
  flex: 1;
}

.buttons {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}
</style>
