<script setup lang="ts">
import { ref, onMounted, useTemplateRef } from "vue";
import { load, type Store } from "@tauri-apps/plugin-store";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useRouter } from "vue-router";
import Icon from "../components/Icon.vue";
import HotkeyInput from "../components/HotkeyInput.vue";
import { getFilename } from "../utils/path";
import { DEFAULT_HOTKEYS, STORE_KEYS, SETTINGS_STORE_FILE } from "../constants";

const router = useRouter();

interface Settings {
  hotkey: string;
  hotkey_de: string;
  hotkey_mute: string;
  model_path: string | null;
  audio_device: string;
  output_mode: "type" | "copy";
}

const settings = ref<Settings>({
  hotkey: DEFAULT_HOTKEYS.ENGLISH,
  hotkey_de: "",
  hotkey_mute: DEFAULT_HOTKEYS.MUTE,
  model_path: null,
  audio_device: "",
  output_mode: "type",
});

const showModelWarning = ref(false);
let store: Store | null = null;

const audioDevices = ref<string[]>([]);
const isRefreshingDevices = ref(false);
const saveError = ref<string | null>(null);
const isSaving = ref(false);

const hotkeyEnRef = useTemplateRef<InstanceType<typeof HotkeyInput>>("hotkeyEnRef");
const hotkeyDeRef = useTemplateRef<InstanceType<typeof HotkeyInput>>("hotkeyDeRef");
const hotkeyMuteRef = useTemplateRef<InstanceType<typeof HotkeyInput>>("hotkeyMuteRef");

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
  store = await load(SETTINGS_STORE_FILE);

  const savedHotkey = await store.get(STORE_KEYS.HOTKEY);
  const savedHotkeyDe = await store.get(STORE_KEYS.HOTKEY_DE);
  const savedHotkeyMute = await store.get(STORE_KEYS.HOTKEY_MUTE);
  const savedModelPath = await store.get(STORE_KEYS.MODEL_PATH);
  const savedAudioDevice = await store.get(STORE_KEYS.AUDIO_DEVICE);
  const savedOutputMode = await store.get(STORE_KEYS.OUTPUT_MODE);

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

  if (savedOutputMode === "type" || savedOutputMode === "copy") {
    settings.value.output_mode = savedOutputMode;
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

function handleRecordingStart(refName: "hotkeyEnRef" | "hotkeyDeRef" | "hotkeyMuteRef") {
  // Stop other recordings
  const refs = { hotkeyEnRef, hotkeyDeRef, hotkeyMuteRef };
  Object.entries(refs).forEach(([name, ref]) => {
    if (name !== refName && ref.value) {
      ref.value.stopRecording();
    }
  });
  invoke("disable_shortcuts").catch((e) => console.error("Failed to disable shortcuts:", e));
}

function handleRecordingEnd() {
  invoke("enable_shortcuts").catch((e) => console.error("Failed to enable shortcuts:", e));
}

async function saveSettings() {
  if (!store || isSaving.value) return;

  isSaving.value = true;
  saveError.value = null;

  try {
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

    await store.set(STORE_KEYS.HOTKEY, settings.value.hotkey);
    await store.set(STORE_KEYS.HOTKEY_DE, settings.value.hotkey_de || "");
    await store.set(STORE_KEYS.HOTKEY_MUTE, settings.value.hotkey_mute || DEFAULT_HOTKEYS.MUTE);
    await store.set(STORE_KEYS.MODEL_PATH, settings.value.model_path);
    await store.set(STORE_KEYS.AUDIO_DEVICE, settings.value.audio_device || "");
    await store.set(STORE_KEYS.OUTPUT_MODE, settings.value.output_mode);
    await store.save();

    try {
      await invoke("reload_settings");
    } catch (e) {
      console.error("Failed to reload settings:", e);
      saveError.value = String(e);
      return;
    }

    router.push("/");
  } finally {
    isSaving.value = false;
  }
}

function cancel() {
  router.push("/");
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
        <select id="audio-device" class="input" v-model="settings.audio_device">
          <option value="">System Default</option>
          <option v-for="device in audioDevices" :key="device" :value="device">
            {{ device }}
          </option>
        </select>
        <button 
          class="btn btn-icon" 
          @click="loadAudioDevices" 
          :disabled="isRefreshingDevices"
          aria-label="Refresh audio devices"
        >
          <Icon name="refresh" :size="14" :class="{ spinning: isRefreshingDevices }" />
        </button>
      </div>
    </section>

    <!-- Output Mode -->
    <section class="section">
      <h2 class="section-title">Output Mode</h2>
      <p class="section-desc">Choose how transcribed text is delivered</p>
      <div class="output-toggle">
        <div class="toggle-track">
          <div class="toggle-slider" :class="{ 'slide-right': settings.output_mode === 'copy' }"></div>
          <button 
            type="button"
            class="toggle-option" 
            :class="{ active: settings.output_mode === 'type' }"
            @click="settings.output_mode = 'type'"
          >
            <svg class="toggle-icon" viewBox="0 0 20 20" fill="currentColor">
              <path d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 5a1 1 0 011-1h6a1 1 0 110 2H4a1 1 0 01-1-1z"/>
              <path d="M15 12l2 2m0 0l2-2m-2 2V8" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            <span class="toggle-label">Type</span>
          </button>
          <button 
            type="button"
            class="toggle-option" 
            :class="{ active: settings.output_mode === 'copy' }"
            @click="settings.output_mode = 'copy'"
          >
            <svg class="toggle-icon" viewBox="0 0 20 20" fill="currentColor">
              <path d="M8 3a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z"/>
              <path d="M6 3a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V5a2 2 0 00-2-2 3 3 0 01-3 3H9a3 3 0 01-3-3z"/>
            </svg>
            <span class="toggle-label">Copy</span>
          </button>
        </div>
        <p class="toggle-hint">
          {{ settings.output_mode === 'type' ? 'Text will be typed at your cursor position' : 'Text will be copied to clipboard and pasted at your cursor position' }}
        </p>
      </div>
    </section>

    <!-- Hotkeys -->
    <section class="section">
      <h2 class="section-title">Hotkeys</h2>
      <p class="section-desc">Configure push-to-talk shortcuts</p>

      <HotkeyInput
        ref="hotkeyEnRef"
        v-model="settings.hotkey"
        label="English"
        @recording-start="handleRecordingStart('hotkeyEnRef')"
        @recording-end="handleRecordingEnd"
      />
      <HotkeyInput
        ref="hotkeyDeRef"
        v-model="settings.hotkey_de"
        label="German"
        :optional="true"
        placeholder="Not set"
        @recording-start="handleRecordingStart('hotkeyDeRef')"
        @recording-end="handleRecordingEnd"
      />
      <HotkeyInput
        ref="hotkeyMuteRef"
        v-model="settings.hotkey_mute"
        label="Mute/Unmute"
        @recording-start="handleRecordingStart('hotkeyMuteRef')"
        @recording-end="handleRecordingEnd"
      />
    </section>

    <!-- Actions -->
    <div class="actions">
      <button class="btn" @click="cancel">Cancel</button>
      <button class="btn btn-primary" @click="saveSettings" :disabled="isSaving">
        {{ isSaving ? "Saving..." : "Save" }}
      </button>
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

.output-toggle {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.toggle-track {
  position: relative;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px;
  padding: 4px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
}

.toggle-slider {
  position: absolute;
  top: 4px;
  left: 4px;
  width: calc(50% - 4px);
  height: calc(100% - 8px);
  background: linear-gradient(135deg, var(--accent) 0%, var(--accent-hover) 100%);
  border-radius: var(--radius-md);
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 8px rgba(20, 184, 166, 0.3);
}

.toggle-slider.slide-right {
  transform: translateX(calc(100% + 4px));
}

.toggle-option {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 16px;
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s var(--ease);
}

.toggle-option .toggle-icon {
  width: 16px;
  height: 16px;
  color: var(--text-muted);
  transition: color 0.2s var(--ease);
}

.toggle-option .toggle-label {
  font-family: var(--font-sans);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  transition: color 0.2s var(--ease);
}

.toggle-option.active .toggle-icon,
.toggle-option.active .toggle-label {
  color: var(--text-inverse);
}

.toggle-option:not(.active):hover .toggle-icon,
.toggle-option:not(.active):hover .toggle-label {
  color: var(--text-primary);
}

.toggle-option:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.toggle-hint {
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
  min-height: 18px;
  transition: color 0.15s var(--ease);
}

.actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding-top: 16px;
  border-top: 1px solid var(--border-light);
}
</style>
