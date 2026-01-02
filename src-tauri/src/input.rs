use enigo::{Enigo, Keyboard, Settings};

use crate::error::Result;

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
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
