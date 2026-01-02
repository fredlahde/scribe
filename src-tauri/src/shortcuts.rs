//! Global shortcut setup and registration.

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::recording::{handle_mute_toggle, handle_recording_start, handle_recording_stop};
use crate::settings::AppSettings;
use crate::transcribe::Language;

/// Setup a recording shortcut for a specific language.
pub fn setup_shortcut(
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

/// Setup the mute toggle shortcut.
pub fn setup_mute_shortcut(
    app: &tauri::AppHandle,
    shortcut_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut: Shortcut = shortcut_str.parse()?;
    let app_handle = app.clone();

    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            // Only toggle on key press, not release
            if event.state == ShortcutState::Pressed {
                handle_mute_toggle(&app_handle);
            }
        })?;

    eprintln!("[Mute shortcut registered: {}]", shortcut_str);
    Ok(())
}

/// Register all shortcuts (English, German if configured, and mute) from settings.
/// This unregisters all existing shortcuts first.
pub fn register_all_shortcuts(
    app: &tauri::AppHandle,
    settings: &AppSettings,
) -> Result<(), String> {
    // Unregister all shortcuts first
    let shortcut_manager = app.global_shortcut();
    shortcut_manager
        .unregister_all()
        .map_err(|e| format!("failed to unregister shortcuts: {}", e))?;

    // Register English shortcut
    if let Err(e) = setup_shortcut(app, &settings.hotkey_en, Language::English) {
        eprintln!("[Failed to setup English shortcut: {}]", e);
        return Err(format!("failed to setup English shortcut: {}", e));
    }

    // Register German shortcut if configured
    if let Some(ref hotkey) = settings.hotkey_de {
        if !hotkey.is_empty() {
            if let Err(e) = setup_shortcut(app, hotkey, Language::German) {
                eprintln!("[Failed to setup German shortcut: {}]", e);
                return Err(format!("failed to setup German shortcut: {}", e));
            }
        }
    }

    // Register mute shortcut
    if let Err(e) = setup_mute_shortcut(app, &settings.hotkey_mute) {
        eprintln!("[Failed to setup mute shortcut: {}]", e);
        return Err(format!("failed to setup mute shortcut: {}", e));
    }

    Ok(())
}
