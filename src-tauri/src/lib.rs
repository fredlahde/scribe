mod audio;
mod error;
mod input;
mod settings;
mod transcribe;
mod tray;

use std::sync::{Arc, Mutex};
use std::thread;

use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;

use crate::audio::AudioRecorder;
use crate::input::TextInput;
use crate::settings::{AppStateHolder, RecordingState};
use crate::transcribe::{Language, Transcriber};
use crate::tray::{create_tray, open_settings_window, update_tray_state, TRAY_ID};

/// Shared app resources
pub struct AppResources {
    pub recorder: AudioRecorder,
    pub transcriber: Option<Transcriber>,
    pub text_input: TextInput,
    pub state: AppStateHolder,
    /// The language to use for the current/next transcription
    pub pending_language: Language,
}

#[tauri::command]
async fn reload_settings(app: tauri::AppHandle) -> Result<(), String> {
    // Load new settings from store
    let store = app
        .store("settings.json")
        .map_err(|e| format!("Failed to open store: {}", e))?;

    let hotkey_en = store
        .get("hotkey")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "F2".to_string());

    let hotkey_de = store
        .get("hotkey_de")
        .and_then(|v| v.as_str().map(String::from));

    let model_path = store
        .get("model_path")
        .and_then(|v| v.as_str().map(String::from));

    // Unregister all shortcuts and re-register with new hotkeys
    let shortcut_manager = app.global_shortcut();
    shortcut_manager
        .unregister_all()
        .map_err(|e| format!("Failed to unregister shortcuts: {}", e))?;

    // Re-register English shortcut
    if let Err(e) = setup_shortcut(&app, &hotkey_en, Language::English) {
        eprintln!("Failed to setup English shortcut: {}", e);
        return Err(format!("Failed to setup shortcut: {}", e));
    }

    // Re-register German shortcut if configured
    if let Some(ref hotkey) = hotkey_de {
        if !hotkey.is_empty() {
            if let Err(e) = setup_shortcut(&app, hotkey, Language::German) {
                eprintln!("Failed to setup German shortcut: {}", e);
                return Err(format!("Failed to setup German shortcut: {}", e));
            }
        }
    }

    // Reload transcriber if model path changed
    if let Some(path) = model_path {
        let resources = app.state::<Arc<Mutex<AppResources>>>();
        let mut res = resources.lock().unwrap();
        match Transcriber::new(&path) {
            Ok(t) => {
                res.transcriber = Some(t);
                eprintln!("[Model loaded: {}]", path);
            }
            Err(e) => {
                eprintln!("[Failed to load model: {}]", e);
                return Err(format!("Failed to load model: {}", e));
            }
        }
    }

    Ok(())
}

fn setup_shortcut(
    app: &tauri::AppHandle,
    shortcut_str: &str,
    language: Language,
) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut: Shortcut = shortcut_str.parse()?;
    let app_handle = app.clone();

    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            let app = app_handle.clone();

            match event.state {
                ShortcutState::Pressed => {
                    handle_recording_start(&app, language);
                }
                ShortcutState::Released => {
                    handle_recording_stop(&app);
                }
            }
        })?;

    eprintln!("[Shortcut registered: {} ({:?})]", shortcut_str, language);
    Ok(())
}

fn handle_recording_start(app: &tauri::AppHandle, language: Language) {
    let resources = app.state::<Arc<Mutex<AppResources>>>();
    let mut res = resources.lock().unwrap();

    // Check if transcriber is loaded
    if res.transcriber.is_none() {
        eprintln!("[No model loaded - opening settings]");
        drop(res);
        open_settings_window(app);
        return;
    }

    // Store the language to use for transcription
    res.pending_language = language;

    // Update state to Recording
    res.state.set(RecordingState::Recording);

    // Update tray icon
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = update_tray_state(&tray, RecordingState::Recording);
    }

    // Start audio recorder
    if let Err(e) = res.recorder.start() {
        eprintln!("[Recording error: {}]", e);
    }
}

fn handle_recording_stop(app: &tauri::AppHandle) {
    let app_clone = app.clone();

    // Spawn a thread to handle transcription (it's blocking)
    thread::spawn(move || {
        let resources = app_clone.state::<Arc<Mutex<AppResources>>>();

        // Stop recording and get samples + language
        let (audio, language) = {
            let res = resources.lock().unwrap();

            // Update state to Transcribing
            res.state.set(RecordingState::Transcribing);

            // Update tray icon
            if let Some(tray) = app_clone.tray_by_id(TRAY_ID) {
                let _ = update_tray_state(&tray, RecordingState::Transcribing);
            }

            let language = res.pending_language;

            match res.recorder.stop() {
                Ok(a) => (a, language),
                Err(e) => {
                    eprintln!("[Stop error: {}]", e);
                    res.state.set(RecordingState::Idle);
                    if let Some(tray) = app_clone.tray_by_id(TRAY_ID) {
                        let _ = update_tray_state(&tray, RecordingState::Idle);
                    }
                    return;
                }
            }
        };

        eprintln!("[Transcribing {} samples ({:?})...]", audio.len(), language);

        if !audio.is_empty() {
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
                    if !text.is_empty() {
                        eprintln!("[Transcribed: {}]", text);

                        // Type the text
                        {
                            let mut res = resources.lock().unwrap();
                            if let Err(e) = res.text_input.type_text(&text) {
                                eprintln!("[Type error: {}]", e);
                            }
                        }

                        // Show notification
                        let _ = app_clone
                            .notification()
                            .builder()
                            .title("Whisper to Me")
                            .body("Transcription complete")
                            .show();
                    } else {
                        eprintln!("[No speech detected]");
                    }
                }
                Err(e) => {
                    eprintln!("[Transcription error: {}]", e);
                }
            }
        } else {
            eprintln!("[No audio captured]");
        }

        // Reset state to Idle
        {
            let res = resources.lock().unwrap();
            res.state.set(RecordingState::Idle);
        }

        // Update tray icon
        if let Some(tray) = app_clone.tray_by_id("main") {
            let _ = update_tray_state(&tray, RecordingState::Idle);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Single instance - must be first
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus settings window if exists, otherwise create it
            if let Some(window) = app.get_webview_window("settings") {
                let _ = window.set_focus();
            } else {
                open_settings_window(app);
            }
        }))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![reload_settings])
        .setup(|app| {
            // Load settings from store
            let store = app.store("settings.json")?;
            let hotkey_en = store
                .get("hotkey")
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "F2".to_string());
            let hotkey_de = store
                .get("hotkey_de")
                .and_then(|v| v.as_str().map(String::from));
            let model_path = store
                .get("model_path")
                .and_then(|v| v.as_str().map(String::from));

            // Initialize audio recorder
            let recorder =
                AudioRecorder::new().map_err(|e| format!("Failed to init audio: {}", e))?;

            // Initialize text input (lazy - actual Enigo init deferred until use)
            let text_input = TextInput::new();

            // Initialize transcriber if model path is set
            let transcriber = if let Some(ref path) = model_path {
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
            if let Err(e) = setup_shortcut(app.handle(), &hotkey_en, Language::English) {
                eprintln!("[Failed to setup English shortcut: {}]", e);
            }

            // Setup German shortcut if configured
            if let Some(ref hotkey) = hotkey_de {
                if !hotkey.is_empty() {
                    if let Err(e) = setup_shortcut(app.handle(), hotkey, Language::German) {
                        eprintln!("[Failed to setup German shortcut: {}]", e);
                    }
                }
            }

            // If no model configured, open settings window
            if model_path.is_none() {
                let window = tauri::WebviewWindowBuilder::new(
                    app,
                    "settings",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .title("Whisper to Me - Settings")
                .inner_size(400.0, 380.0)
                .resizable(false)
                .build()?;

                #[cfg(debug_assertions)]
                window.open_devtools();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
