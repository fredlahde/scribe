//! Application constants and shared values.

/// Overlay window dimensions
pub const OVERLAY_WIDTH: i32 = 200;
pub const OVERLAY_HEIGHT_WARMUP: i32 = 70;
pub const OVERLAY_HEIGHT_RECORDING: i32 = 50;
pub const OVERLAY_BOTTOM_OFFSET: i32 = 60;

/// Warmup timing
pub const WARMUP_MIN_DISPLAY_SECS: u64 = 1;
pub const WARMUP_EMIT_INTERVAL_MS: u64 = 100;
pub const WARMUP_EMIT_COUNT: u8 = 5;

use tauri::{PhysicalPosition, Position, WebviewWindow};

/// Position overlay at bottom center of current monitor.
pub fn position_overlay_bottom_center(overlay: &WebviewWindow, height: i32) -> tauri::Result<()> {
    if let Some(monitor) = overlay.current_monitor()? {
        let size = monitor.size();
        let pos = monitor.position();
        let x = pos.x + (size.width as i32 - OVERLAY_WIDTH) / 2;
        let y = pos.y + size.height as i32 - height - OVERLAY_BOTTOM_OFFSET;
        overlay.set_position(Position::Physical(PhysicalPosition { x, y }))?;
    }
    Ok(())
}
