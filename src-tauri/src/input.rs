use std::process::Command;
use std::thread;
use std::time::Duration;

use enigo::{Enigo, Keyboard, Settings};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::error::{Error, Result};

/// Text input handler that lazily initializes Enigo.
/// This defers the accessibility permission check until text input is actually needed.
pub struct TextInput {
    enigo: Option<Enigo>,
}

impl TextInput {
    pub fn new() -> Self {
        Self { enigo: None }
    }

    /// Ensures Enigo is initialized, creating it on first use.
    fn ensure_enigo(&mut self) -> Result<&mut Enigo> {
        if self.enigo.is_none() {
            self.enigo = Some(Enigo::new(&Settings::default())?);
        }
        Ok(self.enigo.as_mut().unwrap())
    }

    pub fn type_text(&mut self, text: &str) -> Result<()> {
        let enigo = self.ensure_enigo()?;
        enigo.text(text)?;
        Ok(())
    }

    pub fn copy_text(&mut self, app: &tauri::AppHandle, text: &str) -> Result<()> {
        // Copy text to clipboard
        app.clipboard()
            .write_text(text)
            .map_err(|e| Error::Clipboard(e.to_string()))?;

        // Small delay to ensure clipboard is ready
        thread::sleep(Duration::from_millis(50));

        // Simulate Cmd+V to paste at cursor position using AppleScript
        let output = Command::new("osascript")
            .args([
                "-e",
                r#"tell application "System Events" to keystroke "v" using command down"#,
            ])
            .output()
            .map_err(|e| Error::Clipboard(format!("failed to paste: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Clipboard(format!("paste failed: {stderr}")));
        }

        Ok(())
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
