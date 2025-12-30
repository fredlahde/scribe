import { load } from "@tauri-apps/plugin-store";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

let store;
let settings = {
  hotkey: "F2",
  model_path: null,
};
let isRecordingHotkey = false;

async function init() {
  // Load settings from store
  store = await load("settings.json", { autoSave: false });

  const savedHotkey = await store.get("hotkey");
  const savedModelPath = await store.get("model_path");

  if (savedHotkey) settings.hotkey = savedHotkey;
  if (savedModelPath) settings.model_path = savedModelPath;

  updateUI();

  // Show warning if no model
  if (!settings.model_path) {
    document.getElementById("model-warning").style.display = "block";
  }
}

function updateUI() {
  document.getElementById("model-path").value = settings.model_path || "";
  document.getElementById("hotkey-display").value = settings.hotkey;
  document.getElementById("hotkey-status").textContent =
    `Current: ${settings.hotkey}`;
}

// Browse for model file
document.getElementById("browse-btn").addEventListener("click", async (e) => {
  e.preventDefault();
  e.stopPropagation();
  
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
      settings.model_path = selected;
      document.getElementById("model-path").value = selected;
      document.getElementById("model-warning").style.display = "none";
    }
  } catch (err) {
    console.error("Failed to open file dialog:", err);
  }
});

// Record hotkey
document.getElementById("record-hotkey-btn").addEventListener("click", () => {
  const input = document.getElementById("hotkey-display");
  const btn = document.getElementById("record-hotkey-btn");

  if (!isRecordingHotkey) {
    isRecordingHotkey = true;
    input.value = "Press a key...";
    input.focus();
    btn.textContent = "Cancel";
  } else {
    isRecordingHotkey = false;
    input.value = settings.hotkey;
    btn.textContent = "Record";
  }
});

// Capture key press for hotkey
document.getElementById("hotkey-display").addEventListener("keydown", (e) => {
  if (!isRecordingHotkey) return;

  e.preventDefault();

  let parts = [];
  if (e.ctrlKey || e.metaKey) parts.push("CommandOrControl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");

  // Get key name
  let key = e.key;
  if (key === " ") key = "Space";
  if (key.length === 1) key = key.toUpperCase();

  // Skip modifier-only presses
  if (["Control", "Alt", "Shift", "Meta"].includes(key)) return;

  parts.push(key);
  settings.hotkey = parts.join("+");

  document.getElementById("hotkey-display").value = settings.hotkey;
  document.getElementById("record-hotkey-btn").textContent = "Record";
  isRecordingHotkey = false;
});

// Save settings
document.getElementById("save-btn").addEventListener("click", async () => {
  await store.set("hotkey", settings.hotkey);
  await store.set("model_path", settings.model_path);
  await store.save();

  // Notify backend to reload settings
  try {
    await invoke("reload_settings");
  } catch (e) {
    console.error("Failed to reload settings:", e);
  }

  // Close window
  const win = getCurrentWindow();
  await win.close();
});

// Cancel
document.getElementById("cancel-btn").addEventListener("click", async () => {
  const win = getCurrentWindow();
  await win.close();
});

// Initialize
init();
