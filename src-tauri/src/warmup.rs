//! Model warmup logic.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tauri::{Emitter, Manager};

use crate::constants::{
    position_overlay_bottom_center, OVERLAY_HEIGHT_WARMUP, WARMUP_EMIT_COUNT,
    WARMUP_EMIT_INTERVAL_MS, WARMUP_MIN_DISPLAY_SECS,
};
use crate::settings::RecordingState;
use crate::transcribe::Transcriber;
use crate::tray::{update_tray_state, TRAY_ID};
use crate::AppResources;

/// Spawn a background thread to warm up the transcription model.
/// Shows overlay with warmup indicator during the process.
pub fn spawn_warmup(app: &tauri::AppHandle, transcriber: Arc<Transcriber>) {
    let app_handle = app.clone();

    thread::spawn(move || {
        let start_time = Instant::now();

        // Set state to WarmingUp and get hotkeys
        let (hotkey_en, hotkey_mute) = {
            let resources = app_handle.state::<Arc<Mutex<AppResources>>>();
            let res = resources.lock().unwrap();
            res.state.set(RecordingState::WarmingUp);
            (res.hotkey_en.clone(), res.hotkey_mute.clone())
        };

        // Update tray
        if let Some(tray) = app_handle.tray_by_id(TRAY_ID) {
            let _ = update_tray_state(&tray, RecordingState::WarmingUp, &hotkey_en, &hotkey_mute);
        }

        // Show overlay with warmup mode
        show_warmup_overlay(&app_handle);

        // Emit warmup mode multiple times (JS loads asynchronously)
        for _ in 0..WARMUP_EMIT_COUNT {
            let _ = app_handle.emit("overlay-mode", "warmup");
            thread::sleep(Duration::from_millis(WARMUP_EMIT_INTERVAL_MS));
        }

        // Run warmup
        eprintln!("[Warming up model...]");
        match transcriber.warmup() {
            Ok(()) => eprintln!("[Model warmup complete]"),
            Err(e) => eprintln!("[Warmup failed: {e}]"),
        }

        // Ensure minimum display time
        let elapsed = start_time.elapsed();
        if elapsed < Duration::from_secs(WARMUP_MIN_DISPLAY_SECS) {
            thread::sleep(Duration::from_secs(WARMUP_MIN_DISPLAY_SECS) - elapsed);
        }

        // Hide overlay and reset state
        hide_warmup_overlay(&app_handle);
    });
}

fn show_warmup_overlay(app: &tauri::AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = position_overlay_bottom_center(&overlay, OVERLAY_HEIGHT_WARMUP);
        let _ = overlay.show();
    }
}

fn hide_warmup_overlay(app: &tauri::AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.hide();
    }

    let (final_state, hotkey_en, hotkey_mute) = {
        let resources = app.state::<Arc<Mutex<AppResources>>>();
        let res = resources.lock().unwrap();
        let state = if res.state.get() == RecordingState::WarmingUp {
            res.state.set(RecordingState::Idle);
            RecordingState::Idle
        } else {
            res.state.get()
        };
        (state, res.hotkey_en.clone(), res.hotkey_mute.clone())
    };

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = update_tray_state(&tray, final_state, &hotkey_en, &hotkey_mute);
    }
}
