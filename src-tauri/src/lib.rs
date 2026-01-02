mod audio;
mod commands;
mod error;
mod history;
mod input;
mod recording;
mod settings;
mod shortcuts;
mod transcribe;
mod tray;

use std::sync::{Arc, Mutex};

use tauri::Manager;
use tauri_plugin_store::StoreExt;

use crate::audio::AudioRecorder;
use crate::commands::{
    delete_transcription, disable_shortcuts, enable_shortcuts, get_history, list_audio_devices,
    reload_settings, validate_audio_device,
};
use crate::history::HistoryDb;
use crate::input::TextInput;
use crate::settings::{AppSettings, AppStateHolder};
use crate::shortcuts::{setup_mute_shortcut, setup_shortcut};
use crate::transcribe::{Language, Transcriber};
use crate::tray::{create_tray, show_main_window};

/// Shared app resources
pub struct AppResources {
    pub recorder: AudioRecorder,
    pub transcriber: Option<Transcriber>,
    pub text_input: TextInput,
    pub state: AppStateHolder,
    /// The language to use for the current/next transcription
    pub pending_language: Language,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Single instance - must be first
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus main window if exists
            show_main_window(app);
        }))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            reload_settings,
            get_history,
            delete_transcription,
            list_audio_devices,
            validate_audio_device,
            disable_shortcuts,
            enable_shortcuts
        ])
        .setup(|app| {
            // Load settings from store
            let store = app.store("settings.json")?;
            let settings = AppSettings::load(&store);

            // Initialize audio recorder with saved device (or default)
            let recorder = AudioRecorder::new(settings.audio_device.as_deref())
                .map_err(|e| format!("Failed to init audio: {}", e))?;

            // Initialize text input (lazy - actual Enigo init deferred until use)
            let text_input = TextInput::new();

            // Initialize transcriber if model path is set
            let transcriber = if let Some(ref path) = settings.model_path {
                match Transcriber::new(path) {
                    Ok(t) => {
                        eprintln!("[Model loaded: {}]", path);
                        Some(t)
                    }
                    Err(e) => {
                        eprintln!("[Failed to load model: {}]", e);
                        None
                    }
                }
            } else {
                None
            };

            // Initialize history database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to get app data dir: {}", e))?;
            let history_db = HistoryDb::new(app_data_dir)
                .map_err(|e| format!("Failed to init history database: {}", e))?;
            app.manage(Arc::new(history_db));

            // Store resources
            app.manage(Arc::new(Mutex::new(AppResources {
                recorder,
                transcriber,
                text_input,
                state: AppStateHolder::new(),
                pending_language: Language::English,
            })));

            // Create tray with explicit ID
            let _tray = create_tray(app.handle())?;

            // Setup global shortcuts
            if let Err(e) = setup_shortcut(app.handle(), &settings.hotkey_en, Language::English) {
                eprintln!("[Failed to setup English shortcut: {}]", e);
            }

            // Setup German shortcut if configured
            if let Some(ref hotkey) = settings.hotkey_de {
                if !hotkey.is_empty() {
                    if let Err(e) = setup_shortcut(app.handle(), hotkey, Language::German) {
                        eprintln!("[Failed to setup German shortcut: {}]", e);
                    }
                }
            }

            // Setup mute shortcut
            if let Err(e) = setup_mute_shortcut(app.handle(), &settings.hotkey_mute) {
                eprintln!("[Failed to setup mute shortcut: {}]", e);
            }

            // Setup window close handler to hide instead of quit
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent the window from closing, just hide it
                        api.prevent_close();
                        // Check if window is visible before hiding to avoid race conditions
                        match window_clone.is_visible() {
                            Ok(true) => {
                                if let Err(e) = window_clone.hide() {
                                    eprintln!("[Failed to hide window: {}]", e);
                                }
                            }
                            Ok(false) => {
                                // Window already hidden, nothing to do
                            }
                            Err(e) => {
                                eprintln!("[Failed to check window visibility: {}]", e);
                            }
                        }
                    }
                });

                // Open devtools in debug mode
                #[cfg(debug_assertions)]
                window.open_devtools();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
