//! Global shortcut setup and registration.

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::error::{Error, Result};
use crate::recording::{handle_mute_toggle, handle_recording_start, handle_recording_stop};
use crate::settings::AppSettings;
use crate::transcribe::Language;

/// Setup a recording shortcut for a specific language.
pub fn setup_shortcut(
    app: &tauri::AppHandle,
    shortcut_str: &str,
    language: Language,
) -> Result<()> {
    let shortcut: Shortcut = shortcut_str
        .parse()
        .map_err(|e| Error::Hotkey(format!("invalid shortcut '{}': {}", shortcut_str, e)))?;
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
        })
        .map_err(|e| Error::Hotkey(format!("failed to register shortcut: {}", e)))?;

    eprintln!("[Shortcut registered: {} ({:?})]", shortcut_str, language);
    Ok(())
}

/// Setup the mute toggle shortcut.
pub fn setup_mute_shortcut(app: &tauri::AppHandle, shortcut_str: &str) -> Result<()> {
    let shortcut: Shortcut = shortcut_str
        .parse()
        .map_err(|e| Error::Hotkey(format!("invalid shortcut '{}': {}", shortcut_str, e)))?;
    let app_handle = app.clone();

    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            // Only toggle on key press, not release
            if event.state == ShortcutState::Pressed {
                handle_mute_toggle(&app_handle);
            }
        })
        .map_err(|e| Error::Hotkey(format!("failed to register shortcut: {}", e)))?;

    eprintln!("[Mute shortcut registered: {}]", shortcut_str);
    Ok(())
}

/// Register all shortcuts (English, German if configured, and mute) from settings.
/// This unregisters all existing shortcuts first.
pub fn register_all_shortcuts(
    app: &tauri::AppHandle,
    settings: &AppSettings,
) -> std::result::Result<(), String> {
    // Unregister all shortcuts first
    let shortcut_manager = app.global_shortcut();
    shortcut_manager
        .unregister_all()
        .map_err(|e| format!("failed to unregister shortcuts: {}", e))?;

    // Register English shortcut
    setup_shortcut(app, &settings.hotkey_en, Language::English).map_err(|e| e.to_string())?;

    // Register German shortcut if configured
    if let Some(ref hotkey) = settings.hotkey_de {
        if !hotkey.is_empty() {
            setup_shortcut(app, hotkey, Language::German).map_err(|e| e.to_string())?;
        }
    }

    // Register mute shortcut
    setup_mute_shortcut(app, &settings.hotkey_mute).map_err(|e| e.to_string())?;

    Ok(())
}
