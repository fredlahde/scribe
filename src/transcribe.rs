use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::error::{Error, Result};

pub struct Transcriber {
    ctx: WhisperContext,
}

impl Transcriber {
    pub fn new(model_path: &str) -> Result<Self> {
        let mut params = WhisperContextParameters::default();
        params.use_gpu = true;

        let ctx = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| Error::Transcription(format!("failed to load model: {}", e)))?;

        Ok(Self { ctx })
    }

    pub fn transcribe(&self, audio: &[f32]) -> Result<String> {
        if audio.is_empty() {
            return Ok(String::new());
        }

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| Error::Transcription(format!("failed to create state: {}", e)))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Set language to English
        params.set_language(Some("en"));

        // Suppress console output
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, audio)
            .map_err(|e| Error::Transcription(format!("transcription failed: {}", e)))?;

        let num_segments = state.full_n_segments();

        let mut result = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                let text = segment.to_str().map_err(|e| {
                    Error::Transcription(format!("failed to get segment text: {}", e))
                })?;
                result.push_str(text);
            }
        }

        Ok(result.trim().to_string())
    }
}
