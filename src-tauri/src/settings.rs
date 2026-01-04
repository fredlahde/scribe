use std::sync::atomic::{AtomicU8, Ordering};

use tauri_plugin_store::Store;

/// Default hotkey for English transcription
pub const DEFAULT_HOTKEY_EN: &str = "F2";
/// Default hotkey for mute toggle
pub const DEFAULT_HOTKEY_MUTE: &str = "F4";

/// Application settings loaded from the store
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub hotkey_en: String,
    pub hotkey_de: Option<String>,
    pub hotkey_mute: String,
    pub model_path: Option<String>,
    pub audio_device: Option<String>,
}

impl AppSettings {
    /// Load settings from a Tauri store
    pub fn load<R: tauri::Runtime>(store: &Store<R>) -> Self {
        let hotkey_en = store
            .get("hotkey")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| DEFAULT_HOTKEY_EN.to_string());

        let hotkey_de = store
            .get("hotkey_de")
            .and_then(|v| v.as_str().map(String::from));

        let hotkey_mute = store
            .get("hotkey_mute")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| DEFAULT_HOTKEY_MUTE.to_string());

        let model_path = store
            .get("model_path")
            .and_then(|v| v.as_str().map(String::from));

        let audio_device = store
            .get("audio_device")
            .and_then(|v| v.as_str().map(String::from));

        Self {
            hotkey_en,
            hotkey_de,
            hotkey_mute,
            model_path,
            audio_device,
        }
    }
}

/// Application state for tray icon updates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordingState {
    Idle = 0,
    Recording = 1,
    Transcribing = 2,
    Muted = 3,
    WarmingUp = 4,
}

impl From<u8> for RecordingState {
    fn from(val: u8) -> Self {
        match val {
            1 => RecordingState::Recording,
            2 => RecordingState::Transcribing,
            3 => RecordingState::Muted,
            4 => RecordingState::WarmingUp,
            _ => RecordingState::Idle,
        }
    }
}

/// Thread-safe state wrapper
pub struct AppStateHolder {
    pub state: AtomicU8,
}

impl AppStateHolder {
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(RecordingState::Idle as u8),
        }
    }

    pub fn set(&self, state: RecordingState) {
        self.state.store(state as u8, Ordering::SeqCst);
    }

    pub fn get(&self) -> RecordingState {
        RecordingState::from(self.state.load(Ordering::SeqCst))
    }
}

impl Default for AppStateHolder {
    fn default() -> Self {
        Self::new()
    }
}
