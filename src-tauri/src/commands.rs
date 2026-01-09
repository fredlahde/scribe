//! Tauri command handlers for the frontend.

use std::sync::{Arc, Mutex};

use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_store::StoreExt;

use crate::history::{HistoryDb, Transcription};
use crate::settings::AppSettings;
use crate::shortcuts::register_all_shortcuts;
use crate::transcribe::Transcriber;
use crate::warmup;
use crate::AppResources;

#[tauri::command]
pub fn list_audio_devices() -> Vec<String> {
    crate::audio::list_input_devices()
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri commands require owned parameters
pub fn validate_audio_device(device_name: Option<String>) -> bool {
    crate::audio::device_exists(device_name.as_deref())
}

#[tauri::command]
pub async fn disable_shortcuts(app: tauri::AppHandle) -> Result<(), String> {
    let shortcut_manager = app.global_shortcut();
    shortcut_manager
        .unregister_all()
        .map_err(|e| format!("Failed to unregister shortcuts: {e}"))?;
    eprintln!("[Shortcuts disabled for hotkey configuration]");
    Ok(())
}

#[tauri::command]
pub async fn enable_shortcuts(app: tauri::AppHandle) -> Result<(), String> {
    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to open store: {e}"))?;

    let settings = AppSettings::load(&store);
    register_all_shortcuts(&app, &settings)?;

    eprintln!("[Shortcuts re-enabled]");
    Ok(())
}

#[tauri::command]
pub async fn reload_settings(app: tauri::AppHandle) -> Result<(), String> {
    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to open store: {e}"))?;

    let settings = AppSettings::load(&store);

    // Switch audio device if changed
    {
        let resources = app.state::<Arc<Mutex<AppResources>>>();
        let mut res = resources.lock().unwrap();
        if let Err(e) = res.recorder.set_device(settings.audio_device.as_deref()) {
            eprintln!("[Failed to switch audio device: {e}]");
            return Err(format!("Failed to switch audio device: {e}"));
        }
        // Update hotkey settings for tray tooltips
        res.hotkey_en.clone_from(&settings.hotkey_en);
        res.hotkey_mute.clone_from(&settings.hotkey_mute);
        // Update output mode
        res.output_mode = settings.output_mode.clone();
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
                    eprintln!("[Model loaded: {path}]");
                    Some(transcriber)
                }
                Err(e) => {
                    eprintln!("[Failed to load model: {e}]");
                    return Err(format!("Failed to load model: {e}"));
                }
            }
        };

        // Run warmup in background if model was loaded
        if let Some(transcriber) = transcriber {
            warmup::spawn_warmup(&app, transcriber);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_history(app: tauri::AppHandle) -> Result<Vec<Transcription>, String> {
    let history_db = app.state::<Arc<HistoryDb>>();
    history_db
        .get_history(50)
        .map_err(|e| format!("Failed to get history: {e}"))
}

#[tauri::command]
pub async fn delete_transcription(app: tauri::AppHandle, id: i64) -> Result<bool, String> {
    let history_db = app.state::<Arc<HistoryDb>>();
    history_db
        .delete_transcription(id)
        .map_err(|e| format!("Failed to delete transcription: {e}"))
}
