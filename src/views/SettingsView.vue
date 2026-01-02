<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from "vue";
import { load, type Store } from "@tauri-apps/plugin-store";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useRouter } from "vue-router";

const router = useRouter();

interface Settings {
  hotkey: string;
  hotkey_de: string;
  hotkey_mute: string;
  model_path: string | null;
  audio_device: string;
}

const settings = ref<Settings>({
  hotkey: "F2",
  hotkey_de: "",
  hotkey_mute: "F4",
  model_path: null,
  audio_device: "",
});

const isRecordingHotkey = ref(false);
const isRecordingHotkeyDe = ref(false);
const isRecordingHotkeyMute = ref(false);
const showModelWarning = ref(false);
let store: Store | null = null;

const audioDevices = ref<string[]>([]);
const isRefreshingDevices = ref(false);
const saveError = ref<string | null>(null);

const hotkeyInputEn = ref<HTMLInputElement | null>(null);
const hotkeyInputDe = ref<HTMLInputElement | null>(null);
const hotkeyInputMute = ref<HTMLInputElement | null>(null);

watch(isRecordingHotkey, async (recording) => {
  if (recording) {
    await nextTick();
    hotkeyInputEn.value?.focus();
  }
});

watch(isRecordingHotkeyDe, async (recording) => {
  if (recording) {
    await nextTick();
    hotkeyInputDe.value?.focus();
  }
});

watch(isRecordingHotkeyMute, async (recording) => {
  if (recording) {
    await nextTick();
    hotkeyInputMute.value?.focus();
  }
});

async function loadAudioDevices() {
  isRefreshingDevices.value = true;
  try {
    audioDevices.value = await invoke<string[]>("list_audio_devices");
  } catch (err) {
    console.error("Failed to load audio devices:", err);
  } finally {
    isRefreshingDevices.value = false;
  }
}

onMounted(async () => {
  store = await load("settings.json");

  const savedHotkey = await store.get("hotkey");
  const savedHotkeyDe = await store.get("hotkey_de");
  const savedHotkeyMute = await store.get("hotkey_mute");
  const savedModelPath = await store.get("model_path");
  const savedAudioDevice = await store.get("audio_device");

  if (typeof savedHotkey === "string") {
    settings.value.hotkey = savedHotkey;
  }
  if (typeof savedHotkeyDe === "string") {
    settings.value.hotkey_de = savedHotkeyDe;
  }
  if (typeof savedHotkeyMute === "string") {
    settings.value.hotkey_mute = savedHotkeyMute;
  }
  if (typeof savedModelPath === "string") {
    settings.value.model_path = savedModelPath;
  }
  if (typeof savedAudioDevice === "string") {
    settings.value.audio_device = savedAudioDevice;
  }

  showModelWarning.value = !settings.value.model_path;
  await loadAudioDevices();
});

async function browseModel() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Whisper Model", extensions: ["bin"] }],
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
  const wasRecording = isRecordingHotkey.value || isRecordingHotkeyDe.value || isRecordingHotkeyMute.value;
  
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

  const isNowRecording = isRecordingHotkey.value || isRecordingHotkeyDe.value || isRecordingHotkeyMute.value;
  
  // Disable shortcuts when starting to record a hotkey
  if (!wasRecording && isNowRecording) {
    invoke("disable_shortcuts").catch((e) => console.error("Failed to disable shortcuts:", e));
  }
  // Re-enable shortcuts when cancelling (not when a key is captured - that's handled in handleKeydown)
  else if (wasRecording && !isNowRecording) {
    invoke("enable_shortcuts").catch((e) => console.error("Failed to enable shortcuts:", e));
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

  // Re-enable shortcuts after capturing a key
  invoke("enable_shortcuts").catch((e) => console.error("Failed to enable shortcuts:", e));
}

async function saveSettings() {
  if (!store) return;

  saveError.value = null;

  if (settings.value.audio_device) {
    try {
      const isValid = await invoke<boolean>("validate_audio_device", {
        deviceName: settings.value.audio_device,
      });
      if (!isValid) {
        saveError.value = `Audio device "${settings.value.audio_device}" is no longer available.`;
        return;
      }
    } catch (e) {
      console.error("Failed to validate audio device:", e);
      saveError.value = String(e);
      return;
    }
  }

  await store.set("hotkey", settings.value.hotkey);
  await store.set("hotkey_de", settings.value.hotkey_de || "");
  await store.set("hotkey_mute", settings.value.hotkey_mute || "F4");
  await store.set("model_path", settings.value.model_path);
  await store.set("audio_device", settings.value.audio_device || "");
  await store.save();

  try {
    await invoke("reload_settings");
  } catch (e) {
    console.error("Failed to reload settings:", e);
    saveError.value = String(e);
    return;
  }

  router.push("/");
}

function cancel() {
  router.push("/");
}

function getFilename(path: string | null): string {
  if (!path) return "";
  return path.split("/").pop() || path;
}
</script>

<template>
  <div class="settings">
    <div v-if="showModelWarning" class="warning">
      Please select a Whisper model to get started.
    </div>

    <div v-if="saveError" class="error">
      {{ saveError }}
    </div>

    <!-- Model -->
    <section class="section">
      <h2 class="section-title">Whisper Model</h2>
      <p class="section-desc">Select a .bin model file for transcription</p>
      <div class="field-row">
        <div class="model-box" :class="{ empty: !settings.model_path }">
          <span class="model-name">{{ settings.model_path ? getFilename(settings.model_path) : 'No model selected' }}</span>
        </div>
        <button class="btn" @click="browseModel">Browse</button>
      </div>
    </section>

    <!-- Audio Device -->
    <section class="section">
      <h2 class="section-title">Audio Input</h2>
      <p class="section-desc">Choose your microphone</p>
      <div class="field-row">
        <select class="input" v-model="settings.audio_device">
          <option value="">System Default</option>
          <option v-for="device in audioDevices" :key="device" :value="device">
            {{ device }}
          </option>
        </select>
        <button class="btn btn-icon" @click="loadAudioDevices" :disabled="isRefreshingDevices">
          <svg 
            width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" 
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            :class="{ spinning: isRefreshingDevices }"
          >
            <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/>
            <path d="M21 3v5h-5"/>
          </svg>
        </button>
      </div>
    </section>

    <!-- Hotkeys -->
    <section class="section">
      <h2 class="section-title">Hotkeys</h2>
      <p class="section-desc">Configure push-to-talk shortcuts</p>

      <div class="field">
        <label class="field-label">English</label>
        <div class="field-row">
          <input
            ref="hotkeyInputEn"
            type="text"
            class="input hotkey-input"
            :class="{ recording: isRecordingHotkey }"
            :value="isRecordingHotkey ? 'Press a key...' : settings.hotkey"
            readonly
            @keydown="handleKeydown($event, 'en')"
          />
          <button class="btn" @click="startRecordingHotkey('en')">
            {{ isRecordingHotkey ? "Cancel" : "Set" }}
          </button>
        </div>
      </div>

      <div class="field">
        <label class="field-label">German <span class="optional">(optional)</span></label>
        <div class="field-row">
          <input
            ref="hotkeyInputDe"
            type="text"
            class="input hotkey-input"
            :class="{ recording: isRecordingHotkeyDe }"
            :value="isRecordingHotkeyDe ? 'Press a key...' : (settings.hotkey_de || '')"
            readonly
            placeholder="Not set"
            @keydown="handleKeydown($event, 'de')"
          />
          <button class="btn" @click="startRecordingHotkey('de')">
            {{ isRecordingHotkeyDe ? "Cancel" : "Set" }}
          </button>
        </div>
      </div>

      <div class="field">
        <label class="field-label">Mute/Unmute</label>
        <div class="field-row">
          <input
            ref="hotkeyInputMute"
            type="text"
            class="input hotkey-input"
            :class="{ recording: isRecordingHotkeyMute }"
            :value="isRecordingHotkeyMute ? 'Press a key...' : (settings.hotkey_mute || 'F4')"
            readonly
            @keydown="handleKeydown($event, 'mute')"
          />
          <button class="btn" @click="startRecordingHotkey('mute')">
            {{ isRecordingHotkeyMute ? "Cancel" : "Set" }}
          </button>
        </div>
      </div>
    </section>

    <!-- Actions -->
    <div class="actions">
      <button class="btn" @click="cancel">Cancel</button>
      <button class="btn btn-primary" @click="saveSettings">Save</button>
    </div>
  </div>
</template>

<style scoped>
.settings {
  max-width: 400px;
  margin: 0 auto;
}

.section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 4px 0;
}

.section-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0 0 12px 0;
}

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

.model-box {
  flex: 1;
  padding: 10px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
}

.model-box.empty {
  border-style: dashed;
}

.model-name {
  font-size: 13px;
  color: var(--text-primary);
}

.model-box.empty .model-name {
  color: var(--text-muted);
}

.hotkey-input {
  font-family: var(--font-mono);
  font-size: 13px;
}

.hotkey-input.recording {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-muted);
}

.spinning {
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding-top: 16px;
  border-top: 1px solid var(--border-light);
}
</style>
