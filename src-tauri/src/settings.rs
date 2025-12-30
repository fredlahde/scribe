use std::sync::atomic::{AtomicU8, Ordering};

/// Application state for tray icon updates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordingState {
    Idle = 0,
    Recording = 1,
    Transcribing = 2,
}

impl From<u8> for RecordingState {
    fn from(val: u8) -> Self {
        match val {
            1 => RecordingState::Recording,
            2 => RecordingState::Transcribing,
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
