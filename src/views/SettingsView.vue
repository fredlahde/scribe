<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from "vue";
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
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let store: any = null;

// Audio devices
const audioDevices = ref<string[]>([]);
const isRefreshingDevices = ref(false);
const saveError = ref<string | null>(null);

// Template refs for hotkey inputs
const hotkeyInputEn = ref<HTMLInputElement | null>(null);
const hotkeyInputDe = ref<HTMLInputElement | null>(null);
const hotkeyInputMute = ref<HTMLInputElement | null>(null);

// Watch recording states to manage focus
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

  // Explicitly handle empty strings vs null/undefined
  // A saved empty string means the user cleared the hotkey intentionally
  if (typeof savedHotkey === "string") {
    settings.value.hotkey = savedHotkey;
  }
  if (typeof savedHotkeyDe === "string") {
    settings.value.hotkey_de = savedHotkeyDe;
  }
  if (typeof savedHotkeyMute === "string") {
    settings.value.hotkey_mute = savedHotkeyMute;
  }
  if (savedModelPath) settings.value.model_path = savedModelPath;
  if (typeof savedAudioDevice === "string") {
    // Preserve empty string to represent the default audio device, matching backend behavior
    settings.value.audio_device = savedAudioDevice;
  }

  showModelWarning.value = !settings.value.model_path;

  // Load available audio devices
  await loadAudioDevices();
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

  saveError.value = null;

  // Validate audio device before saving (skip validation for system default)
  if (settings.value.audio_device) {
    try {
      const isValid = await invoke<boolean>("validate_audio_device", {
        deviceName: settings.value.audio_device,
      });
      if (!isValid) {
        saveError.value = `Audio device "${settings.value.audio_device}" is no longer available. Please select a different device.`;
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
</script>

<template>
  <div class="settings-view">
    <!-- Warning banner when no model configured -->
    <div v-if="showModelWarning" class="warning">
      Please select a Whisper model file to get started.
    </div>

    <!-- Error banner for save errors -->
    <div v-if="saveError" class="error">
      Failed to save settings: {{ saveError }}
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

    <!-- Audio Input Device -->
    <div class="setting">
      <label class="setting-label">Audio Input Device:</label>
      <div class="input-row">
        <select class="input" v-model="settings.audio_device">
          <option value="">System Default</option>
          <option v-for="device in audioDevices" :key="device" :value="device">
            {{ device }}
          </option>
        </select>
        <button class="btn" @click="loadAudioDevices" :disabled="isRefreshingDevices">
          {{ isRefreshingDevices ? "..." : "Refresh" }}
        </button>
      </div>
      <small class="setting-hint">Select which microphone to use for recording</small>
    </div>

    <!-- English Hotkey -->
    <div class="setting">
      <label class="setting-label">Push-to-Talk Hotkey (English):</label>
      <div class="input-row">
        <input
          ref="hotkeyInputEn"
          type="text"
          class="input"
          :value="isRecordingHotkey ? 'Press a key...' : settings.hotkey"
          readonly
          tabindex="0"
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
          ref="hotkeyInputDe"
          type="text"
          class="input"
          :value="isRecordingHotkeyDe ? 'Press a key...' : (settings.hotkey_de || '')"
          readonly
          tabindex="0"
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
          ref="hotkeyInputMute"
          type="text"
          class="input"
          :value="isRecordingHotkeyMute ? 'Press a key...' : (settings.hotkey_mute || 'F4')"
          readonly
          tabindex="0"
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
