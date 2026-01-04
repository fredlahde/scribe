use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Runtime,
};

use crate::settings::RecordingState;

pub const TRAY_ID: &str = "main";

pub fn create_tray<R: Runtime>(app: &AppHandle<R>, hotkey_en: &str) -> tauri::Result<TrayIcon<R>> {
    let open_i = MenuItem::with_id(app, "open", "Open Scribe", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit Scribe", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_i, &quit_i])?;

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(load_tray_icon(RecordingState::Idle)?)
        .menu(&menu)
        .tooltip(format!("Scribe - Ready (Press {hotkey_en} to record)"))
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                show_main_window(app);
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
    use std::io::Cursor;

    let icon_bytes: &[u8] = match state {
        RecordingState::Idle => include_bytes!("../icons/tray-idle.png"),
        RecordingState::Recording => include_bytes!("../icons/tray-recording.png"),
        RecordingState::Transcribing => include_bytes!("../icons/tray-transcribing.png"),
        RecordingState::Muted => include_bytes!("../icons/tray-muted.png"),
        RecordingState::WarmingUp => include_bytes!("../icons/tray-warmup.png"),
    };

    // Decode PNG to RGBA - wrap in Cursor to provide Seek trait
    let decoder = png::Decoder::new(Cursor::new(icon_bytes));
    let mut reader = decoder
        .read_info()
        .map_err(|e| tauri::Error::Io(std::io::Error::other(format!("PNG decode error: {e}"))))?;

    let buf_size = reader
        .output_buffer_size()
        .ok_or_else(|| tauri::Error::Io(std::io::Error::other("PNG output buffer size unknown")))?;
    let mut buf = vec![0; buf_size];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| tauri::Error::Io(std::io::Error::other(format!("PNG frame error: {e}"))))?;

    // Truncate to actual size
    buf.truncate(info.buffer_size());

    Ok(Image::new_owned(buf, info.width, info.height))
}

pub fn update_tray_state<R: Runtime>(
    tray: &TrayIcon<R>,
    state: RecordingState,
    hotkey_en: &str,
    hotkey_mute: &str,
) -> tauri::Result<()> {
    let tooltip = match state {
        RecordingState::Idle => format!("Scribe - Ready (Press {hotkey_en} to record)"),
        RecordingState::Recording => "Scribe - Recording...".to_string(),
        RecordingState::Transcribing => "Scribe - Transcribing...".to_string(),
        RecordingState::Muted => format!("Scribe - Muted (Press {hotkey_mute} to unmute)"),
        RecordingState::WarmingUp => "Scribe - Starting up...".to_string(),
    };

    tray.set_tooltip(Some(&tooltip))?;
    tray.set_icon(Some(load_tray_icon(state)?))?;

    Ok(())
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.show() {
            eprintln!("[Failed to show main window: {e}]");
        }
        if let Err(e) = window.set_focus() {
            eprintln!("[Failed to focus main window: {e}]");
        }
    }
}
