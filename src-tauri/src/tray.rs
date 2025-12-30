use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Runtime, WebviewWindowBuilder,
};

use crate::settings::RecordingState;

pub const TRAY_ID: &str = "main";

pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<TrayIcon<R>> {
    let settings_i = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit Whisper to Me", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_i, &quit_i])?;

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(load_tray_icon(RecordingState::Idle)?)
        .menu(&menu)
        .tooltip("Whisper to Me - Ready (Press F2 to record)")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                open_settings_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(tray)
}

pub fn load_tray_icon(state: RecordingState) -> tauri::Result<Image<'static>> {
    let icon_bytes: &[u8] = match state {
        RecordingState::Idle => include_bytes!("../icons/tray-idle.png"),
        RecordingState::Recording => include_bytes!("../icons/tray-recording.png"),
        RecordingState::Transcribing => include_bytes!("../icons/tray-transcribing.png"),
        RecordingState::Muted => include_bytes!("../icons/tray-muted.png"),
    };

    // Decode PNG to RGBA
    let decoder = png::Decoder::new(icon_bytes);
    let mut reader = decoder
        .read_info()
        .map_err(|e| tauri::Error::Io(std::io::Error::other(format!("PNG decode error: {}", e))))?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| tauri::Error::Io(std::io::Error::other(format!("PNG frame error: {}", e))))?;

    // Truncate to actual size
    buf.truncate(info.buffer_size());

    Ok(Image::new_owned(buf, info.width, info.height))
}

pub fn update_tray_state<R: Runtime>(
    tray: &TrayIcon<R>,
    state: RecordingState,
) -> tauri::Result<()> {
    let tooltip = match state {
        RecordingState::Idle => "Whisper to Me - Ready",
        RecordingState::Recording => "Whisper to Me - Recording...",
        RecordingState::Transcribing => "Whisper to Me - Transcribing...",
        RecordingState::Muted => "Whisper to Me - Muted (Press F4 to unmute)",
    };

    tray.set_tooltip(Some(tooltip))?;
    tray.set_icon(Some(load_tray_icon(state)?))?;

    Ok(())
}

pub fn open_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.set_focus();
    } else {
        // Create settings window with devtools enabled in debug builds
        if let Ok(window) =
            WebviewWindowBuilder::new(app, "settings", tauri::WebviewUrl::App("index.html".into()))
                .title("Whisper to Me - Settings")
                .inner_size(400.0, 380.0)
                .resizable(false)
                .build()
        {
            // Open devtools automatically in debug builds
            #[cfg(debug_assertions)]
            window.open_devtools();
        }
    }
}
