use enigo::{Enigo, Keyboard, Settings};

use crate::error::Result;

pub struct TextInput {
    enigo: Enigo,
}

impl TextInput {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default())?;
        Ok(Self { enigo })
    }

    pub fn type_text(&mut self, text: &str) -> Result<()> {
        self.enigo.text(text)?;
        Ok(())
    }
}
