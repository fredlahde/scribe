//! Recording and transcription handling.

use std::sync::{Arc, Mutex};
use std::thread;

use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::constants::{position_overlay_bottom_center, OVERLAY_HEIGHT_RECORDING};
use crate::history::HistoryDb;
use crate::settings::{OutputMode, RecordingState};
use crate::transcribe::Language;
use crate::tray::{show_main_window, update_tray_state, TRAY_ID};
use crate::AppResources;

/// Update tray icon to reflect the given state.
fn set_tray_state(
    app: &tauri::AppHandle,
    state: RecordingState,
    hotkey_en: &str,
    hotkey_mute: &str,
) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = update_tray_state(&tray, state, hotkey_en, hotkey_mute);
    }
}

/// Toggle mute state for the microphone.
pub fn handle_mute_toggle(app: &tauri::AppHandle) {
    let resources = app.state::<Arc<Mutex<AppResources>>>();
    let mut res = resources.lock().unwrap();

    // Extract hotkeys while we have the lock
    let hotkey_en = res.hotkey_en.clone();
    let hotkey_mute = res.hotkey_mute.clone();

    if res.recorder.is_muted() {
        // Unmute
        if let Err(e) = res.recorder.unmute() {
            eprintln!("[Unmute error: {e}]");
            return;
        }
        res.state.set(RecordingState::Idle);
        drop(res); // Release lock before tray update

        // Update tray icon
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = update_tray_state(&tray, RecordingState::Idle, &hotkey_en, &hotkey_mute);
        }

        // Show notification
        let _ = app
            .notification()
            .builder()
            .title("Scribe")
            .body("Microphone enabled")
            .show();
    } else {
        // Mute
        res.recorder.mute();
        res.state.set(RecordingState::Muted);
        drop(res); // Release lock before tray update

        // Update tray icon
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = update_tray_state(&tray, RecordingState::Muted, &hotkey_en, &hotkey_mute);
        }

        // Show notification
        let _ = app
            .notification()
            .builder()
            .title("Scribe")
            .body("Microphone muted")
            .show();
    }
}

/// Start recording audio for the given language.
pub fn handle_recording_start(app: &tauri::AppHandle, language: Language) {
    let resources = app.state::<Arc<Mutex<AppResources>>>();
    let mut res = resources.lock().unwrap();

    // Extract hotkeys early since we need them for early return notifications
    let hotkey_en = res.hotkey_en.clone();
    let hotkey_mute = res.hotkey_mute.clone();

    // Check if warming up
    if res.state.get() == RecordingState::WarmingUp {
        eprintln!("[Cannot record - model is still warming up]");
        drop(res);
        let _ = app
            .notification()
            .builder()
            .title("Scribe")
            .body("Model is starting up, please wait...")
            .show();
        return;
    }

    // Check if muted
    if res.recorder.is_muted() {
        eprintln!("[Cannot record - microphone is muted]");
        drop(res);
        let _ = app
            .notification()
            .builder()
            .title("Scribe")
            .body(format!(
                "Microphone is muted. Press {hotkey_mute} to unmute."
            ))
            .show();
        return;
    }

    // Check if transcriber is loaded
    if res.transcriber.is_none() {
        eprintln!("[No model loaded - opening main window]");
        drop(res);
        show_main_window(app);
        return;
    }

    // Store the language to use for transcription
    res.pending_language = language;

    // Update state to Recording
    res.state.set(RecordingState::Recording);

    // Start audio recorder
    res.recorder.start();

    // Release lock before tray/overlay operations
    drop(res);

    // Update tray icon
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = update_tray_state(&tray, RecordingState::Recording, &hotkey_en, &hotkey_mute);
    }

    // Show overlay window
    if let Some(overlay) = app.get_webview_window("overlay") {
        // Position at bottom center of screen (accounting for monitor position in multi-monitor setups)
        let _ = position_overlay_bottom_center(&overlay, OVERLAY_HEIGHT_RECORDING);
        let _ = overlay.show();
    }

    // Set overlay to waveform mode
    let _ = app.emit("overlay-mode", "waveform");

    // Spawn thread to emit audio levels
    let app_clone = app.clone();
    thread::spawn(move || loop {
        let resources = app_clone.state::<Arc<Mutex<AppResources>>>();
        let res = resources.lock().unwrap();
        let is_recording = res.state.get() == RecordingState::Recording;
        let level = res.recorder.get_audio_level();
        drop(res);

        if !is_recording {
            break;
        }

        let _ = app_clone.emit("audio-level", level);
        thread::sleep(std::time::Duration::from_millis(50));
    });
}

/// Process transcription result: save to history, type text, and notify user.
fn process_transcription_result(
    app: &tauri::AppHandle,
    resources: &Arc<Mutex<AppResources>>,
    text: &str,
    language: Language,
    sample_count: usize,
) {
    if text.is_empty() {
        eprintln!("[No speech detected]");
        return;
    }

    eprintln!("[Transcribed: {} chars]", text.len());

    // Save to history database
    let lang_str = match language {
        Language::English => "en",
        Language::German => "de",
    };
    let history_db = app.state::<Arc<HistoryDb>>();
    match history_db.save_transcription(text, lang_str, sample_count) {
        Ok(record) => {
            eprintln!("[Saved to history: id={}]", record.id);
            // Emit event for frontend to update
            let _ = app.emit("transcription-added", &record);
        }
        Err(e) => {
            eprintln!("[Failed to save to history: {e}]");
        }
    }

    // Output text based on mode
    let (output_mode, output_result) = {
        let mut res = resources.lock().unwrap();
        let mode = res.output_mode.clone();
        let result = match mode {
            OutputMode::Copy => res.text_input.copy_text(app, text),
            OutputMode::Type => res.text_input.type_text(text),
        };
        (mode, result)
    };

    // Show notification based on result
    let body = match &output_result {
        Ok(()) => match output_mode {
            OutputMode::Copy => "Copied and pasted",
            OutputMode::Type => "Transcription complete",
        },
        Err(e) => {
            eprintln!("[Output error: {e}]");
            match output_mode {
                OutputMode::Copy => "Failed to copy and paste",
                OutputMode::Type => "Failed to type text",
            }
        }
    };
    let _ = app
        .notification()
        .builder()
        .title("Scribe")
        .body(body)
        .show();
}

/// Stop recording, run transcription, and handle the result.
/// This runs in a background thread.
fn run_transcription(app: &tauri::AppHandle) {
    let resources = app.state::<Arc<Mutex<AppResources>>>();

    // Stop recording and get samples + language + hotkeys
    let (audio, language, hotkey_en, hotkey_mute) = {
        let res = resources.lock().unwrap();

        // Extract hotkeys while we have the lock
        let hotkey_en = res.hotkey_en.clone();
        let hotkey_mute = res.hotkey_mute.clone();

        // Update state to Transcribing
        res.state.set(RecordingState::Transcribing);

        let language = res.pending_language;

        match res.recorder.stop() {
            Ok(a) => (a, language, hotkey_en, hotkey_mute),
            Err(e) => {
                eprintln!("[Stop error: {e}]");
                res.state.set(RecordingState::Idle);
                drop(res); // Release lock before tray update
                set_tray_state(app, RecordingState::Idle, &hotkey_en, &hotkey_mute);
                return;
            }
        }
    };

    // Update tray after releasing lock
    set_tray_state(app, RecordingState::Transcribing, &hotkey_en, &hotkey_mute);

    eprintln!("[Transcribing {} samples ({language:?})...]", audio.len());

    if audio.is_empty() {
        eprintln!("[No audio captured]");
    } else {
        let sample_count = audio.len();

        // Transcribe
        let transcription = {
            let res = resources.lock().unwrap();
            if let Some(ref transcriber) = res.transcriber {
                transcriber.transcribe(&audio, language)
            } else {
                Ok(String::new())
            }
        };

        match transcription {
            Ok(text) => {
                process_transcription_result(app, &resources, &text, language, sample_count);
            }
            Err(e) => {
                eprintln!("[Transcription error: {e}]");
            }
        }
    }

    // Reset state to Idle
    {
        let res = resources.lock().unwrap();
        res.state.set(RecordingState::Idle);
    }
    set_tray_state(app, RecordingState::Idle, &hotkey_en, &hotkey_mute);

    // Hide overlay after transcription completes
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.hide();
    }
}

/// Stop recording and spawn transcription thread.
pub fn handle_recording_stop(app: &tauri::AppHandle) {
    // Check if we're actually recording before spawning the thread
    {
        let resources = app.state::<Arc<Mutex<AppResources>>>();
        let res = resources.lock().unwrap();
        if res.state.get() != RecordingState::Recording {
            // Not recording, nothing to do
            return;
        }
    }

    // Switch overlay to spinner mode when hotkey is released
    let _ = app.emit("overlay-mode", "spinner");

    // Spawn a thread to handle transcription (it's blocking)
    let app_clone = app.clone();
    thread::spawn(move || {
        run_transcription(&app_clone);
    });
}
