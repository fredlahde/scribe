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
use std::thread;
use std::time::{Duration, Instant};

use tauri::{Emitter, Manager};
use tauri_plugin_store::StoreExt;

use crate::audio::AudioRecorder;
use crate::commands::{
    delete_transcription, disable_shortcuts, enable_shortcuts, get_history, list_audio_devices,
    reload_settings, validate_audio_device,
};
use crate::history::HistoryDb;
use crate::input::TextInput;
use crate::settings::{AppSettings, AppStateHolder, RecordingState};
use crate::shortcuts::{setup_mute_shortcut, setup_shortcut};
use crate::transcribe::{Language, Transcriber};
use crate::tray::{create_tray, show_main_window, update_tray_state, TRAY_ID};

/// Shared app resources
pub struct AppResources {
    pub recorder: AudioRecorder,
    pub transcriber: Option<Arc<Transcriber>>,
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
                        Some(Arc::new(t))
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

            // Setup overlay window
            if let Some(overlay) = app.get_webview_window("overlay") {
                // Make sure it's hidden initially
                let _ = overlay.hide();

                // Open devtools in debug mode
                #[cfg(debug_assertions)]
                overlay.open_devtools();
            }

            // Start warmup in background if model was loaded
            let has_transcriber = {
                let resources = app.state::<Arc<Mutex<AppResources>>>();
                let res = resources.lock().unwrap();
                res.transcriber.is_some()
            };

            if has_transcriber {
                let app_handle = app.handle().clone();
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
                    // (the JS module loads asynchronously)
                    for _ in 0..5 {
                        let _ = app_handle.emit("overlay-mode", "warmup");
                        thread::sleep(Duration::from_millis(100));
                    }

                    // Run warmup (clone Arc to release lock before blocking inference)
                    eprintln!("[Warming up model...]");
                    let transcriber_clone = {
                        let resources = app_handle.state::<Arc<Mutex<AppResources>>>();
                        let res = resources.lock().unwrap();
                        res.transcriber.clone()
                    };
                    if let Some(transcriber) = transcriber_clone {
                        match transcriber.warmup() {
                            Ok(()) => eprintln!("[Model warmup complete]"),
                            Err(e) => eprintln!("[Warmup failed: {}]", e),
                        }
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
                        // Only reset to Idle if we're still in WarmingUp state
                        // (user may have toggled mute during warmup)
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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
