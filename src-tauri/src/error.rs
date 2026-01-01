use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("audio error: {0}")]
    Audio(String),

    #[error("database error: {0}")]
    Database(String),

    #[allow(dead_code)]
    #[error("hotkey error: {0}")]
    Hotkey(String),

    #[error("input error: {0}")]
    Input(#[from] enigo::InputError),

    #[error("input creation error: {0}")]
    InputCreation(#[from] enigo::NewConError),

    #[error("resampling error: {0}")]
    Resample(String),

    #[error("transcription error: {0}")]
    Transcription(String),
}

pub type Result<T> = std::result::Result<T, Error>;
