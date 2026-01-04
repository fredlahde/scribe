mod audio;
mod commands;
mod constants;
mod error;
mod history;
mod input;
mod recording;
mod settings;
mod shortcuts;
mod transcribe;
mod tray;
mod warmup;

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
    pub transcriber: Option<Arc<Transcriber>>,
    pub text_input: TextInput,
    pub state: AppStateHolder,
    /// The language to use for the current/next transcription
    pub pending_language: Language,
    /// Hotkey settings for tray tooltip
    pub hotkey_en: String,
    pub hotkey_mute: String,
}

/// Initialize audio recorder with optional device.
fn init_audio_recorder(settings: &AppSettings) -> Result<AudioRecorder, String> {
    AudioRecorder::new(settings.audio_device.as_deref())
        .map_err(|e| format!("Failed to init audio: {}", e))
}

/// Initialize transcriber from model path.
fn init_transcriber(model_path: Option<&str>) -> Option<Arc<Transcriber>> {
    let path = model_path?;
    match Transcriber::new(path) {
        Ok(t) => {
            eprintln!("[Model loaded: {}]", path);
            Some(Arc::new(t))
        }
        Err(e) => {
            eprintln!("[Failed to load model: {}]", e);
            None
        }
    }
}

/// Initialize history database.
fn init_history_db(app: &tauri::App) -> Result<Arc<HistoryDb>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    HistoryDb::new(app_data_dir)
        .map(Arc::new)
        .map_err(|e| format!("Failed to init history database: {}", e))
}

/// Setup window close handler to hide window instead of quitting.
fn setup_window_close_handler(window: &tauri::WebviewWindow) {
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
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
}

/// Register all global shortcuts from settings.
fn register_shortcuts(app: &tauri::AppHandle, settings: &AppSettings) {
    if let Err(e) = setup_shortcut(app, &settings.hotkey_en, Language::English) {
        eprintln!("[Failed to setup English shortcut: {}]", e);
    }

    if let Some(ref hotkey) = settings.hotkey_de {
        if !hotkey.is_empty() {
            if let Err(e) = setup_shortcut(app, hotkey, Language::German) {
                eprintln!("[Failed to setup German shortcut: {}]", e);
            }
        }
    }

    if let Err(e) = setup_mute_shortcut(app, &settings.hotkey_mute) {
        eprintln!("[Failed to setup mute shortcut: {}]", e);
    }
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

            // Initialize components
            let recorder = init_audio_recorder(&settings)?;
            let transcriber = init_transcriber(settings.model_path.as_deref());
            let history_db = init_history_db(app)?;

            // Manage state
            app.manage(history_db);
            app.manage(Arc::new(Mutex::new(AppResources {
                recorder,
                transcriber: transcriber.clone(),
                text_input: TextInput::new(),
                state: AppStateHolder::new(),
                pending_language: Language::English,
                hotkey_en: settings.hotkey_en.clone(),
                hotkey_mute: settings.hotkey_mute.clone(),
            })));

            // Setup tray and shortcuts
            let _tray = create_tray(app.handle(), &settings.hotkey_en)?;
            register_shortcuts(app.handle(), &settings);

            // Setup main window
            if let Some(window) = app.get_webview_window("main") {
                setup_window_close_handler(&window);
                #[cfg(debug_assertions)]
                window.open_devtools();
            }

            // Setup overlay window
            if let Some(overlay) = app.get_webview_window("overlay") {
                let _ = overlay.hide();
                #[cfg(debug_assertions)]
                overlay.open_devtools();
            }

            // Start warmup in background if model was loaded
            if let Some(t) = transcriber {
                warmup::spawn_warmup(app.handle(), t);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
