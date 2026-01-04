//! Tauri command handlers for the frontend.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_store::StoreExt;

use crate::history::{HistoryDb, Transcription};
use crate::settings::{AppSettings, RecordingState};
use crate::shortcuts::register_all_shortcuts;
use crate::transcribe::Transcriber;
use crate::tray::{update_tray_state, TRAY_ID};
use crate::AppResources;

#[tauri::command]
pub fn list_audio_devices() -> Vec<String> {
    crate::audio::list_input_devices()
}

#[tauri::command]
pub fn validate_audio_device(device_name: Option<String>) -> bool {
    crate::audio::device_exists(device_name.as_deref())
}

#[tauri::command]
pub async fn disable_shortcuts(app: tauri::AppHandle) -> Result<(), String> {
    let shortcut_manager = app.global_shortcut();
    shortcut_manager
        .unregister_all()
        .map_err(|e| format!("Failed to unregister shortcuts: {}", e))?;
    eprintln!("[Shortcuts disabled for hotkey configuration]");
    Ok(())
}

#[tauri::command]
pub async fn enable_shortcuts(app: tauri::AppHandle) -> Result<(), String> {
    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let settings = AppSettings::load(&store);
    register_all_shortcuts(&app, &settings)?;

    eprintln!("[Shortcuts re-enabled]");
    Ok(())
}

#[tauri::command]
pub async fn reload_settings(app: tauri::AppHandle) -> Result<(), String> {
    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let settings = AppSettings::load(&store);

    // Switch audio device if changed
    {
        let resources = app.state::<Arc<Mutex<AppResources>>>();
        let mut res = resources.lock().unwrap();
        if let Err(e) = res.recorder.set_device(settings.audio_device.as_deref()) {
            eprintln!("[Failed to switch audio device: {}]", e);
            return Err(format!("Failed to switch audio device: {}", e));
        }
    }

    // Re-register all shortcuts with new hotkeys
    register_all_shortcuts(&app, &settings)?;

    // Reload transcriber if model path changed
    if let Some(ref path) = settings.model_path {
        let transcriber = {
            let resources = app.state::<Arc<Mutex<AppResources>>>();
            let mut res = resources.lock().unwrap();
            match Transcriber::new(path) {
                Ok(t) => {
                    let transcriber = Arc::new(t);
                    res.transcriber = Some(transcriber.clone());
                    eprintln!("[Model loaded: {}]", path);
                    Some(transcriber)
                }
                Err(e) => {
                    eprintln!("[Failed to load model: {}]", e);
                    return Err(format!("Failed to load model: {}", e));
                }
            }
        };

        // Run warmup in background if model was loaded
        if let Some(transcriber) = transcriber {
            let app_handle = app.clone();
            thread::spawn(move || {
                let start_time = Instant::now();

                // Set state to WarmingUp
                {
                    let resources = app_handle.state::<Arc<Mutex<AppResources>>>();
                    resources
                        .lock()
                        .unwrap()
                        .state
                        .set(RecordingState::WarmingUp);
                }

                // Update tray
                if let Some(tray) = app_handle.tray_by_id(TRAY_ID) {
                    let _ = update_tray_state(&tray, RecordingState::WarmingUp);
                }

                // Show overlay with warmup mode
                if let Some(overlay) = app_handle.get_webview_window("overlay") {
                    if let Ok(Some(monitor)) = overlay.current_monitor() {
                        let size = monitor.size();
                        let position = monitor.position();
                        let overlay_width = 200;
                        let overlay_height = 70;
                        let x = position.x + (size.width as i32 - overlay_width) / 2;
                        let y = position.y + size.height as i32 - overlay_height - 60;
                        let _ = overlay.set_position(tauri::Position::Physical(
                            tauri::PhysicalPosition { x, y },
                        ));
                    }
                    let _ = overlay.show();
                }

                // Emit warmup mode multiple times to ensure the overlay receives it
                for _ in 0..5 {
                    let _ = app_handle.emit("overlay-mode", "warmup");
                    thread::sleep(Duration::from_millis(100));
                }

                // Run warmup
                eprintln!("[Warming up model...]");
                match transcriber.warmup() {
                    Ok(()) => eprintln!("[Model warmup complete]"),
                    Err(e) => eprintln!("[Warmup failed: {}]", e),
                }

                // Ensure minimum 1 second display time
                let elapsed = start_time.elapsed();
                if elapsed < Duration::from_secs(1) {
                    thread::sleep(Duration::from_secs(1) - elapsed);
                }

                // Hide overlay and reset state (only if still warming up)
                if let Some(overlay) = app_handle.get_webview_window("overlay") {
                    let _ = overlay.hide();
                }
                let final_state = {
                    let resources = app_handle.state::<Arc<Mutex<AppResources>>>();
                    let res = resources.lock().unwrap();
                    if res.state.get() == RecordingState::WarmingUp {
                        res.state.set(RecordingState::Idle);
                        RecordingState::Idle
                    } else {
                        res.state.get()
                    }
                };
                if let Some(tray) = app_handle.tray_by_id(TRAY_ID) {
                    let _ = update_tray_state(&tray, final_state);
                }
            });
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_history(app: tauri::AppHandle) -> Result<Vec<Transcription>, String> {
    let history_db = app.state::<Arc<HistoryDb>>();
    history_db
        .get_history(50)
        .map_err(|e| format!("Failed to get history: {}", e))
}

#[tauri::command]
pub async fn delete_transcription(app: tauri::AppHandle, id: i64) -> Result<bool, String> {
    let history_db = app.state::<Arc<HistoryDb>>();
    history_db
        .delete_transcription(id)
        .map_err(|e| format!("Failed to delete transcription: {}", e))
}
