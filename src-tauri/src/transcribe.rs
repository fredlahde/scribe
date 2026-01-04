use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::error::{Error, Result};

/// Sample rate required by Whisper (16kHz)
pub const WHISPER_SAMPLE_RATE: u32 = 16000;

#[derive(Debug, Clone, Copy)]
pub enum Language {
    English,
    German,
}

pub struct Transcriber {
    ctx: WhisperContext,
}

impl Transcriber {
    pub fn new(model_path: &str) -> Result<Self> {
        let params = WhisperContextParameters {
            use_gpu: true,
            ..Default::default()
        };

        let ctx = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| Error::Transcription(format!("failed to load model: {e}")))?;

        Ok(Self { ctx })
    }

    /// Run a brief inference to pre-initialize the compute engine (ANE/CoreML/Metal).
    /// This makes the first real transcription much faster.
    pub fn warmup(&self) -> Result<()> {
        // 0.2 seconds of silence - Whisper requires at least 100ms of audio
        let dummy_audio = vec![0.0f32; (WHISPER_SAMPLE_RATE / 5) as usize];

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| Error::Transcription(format!("warmup state creation failed: {e}")))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, &dummy_audio)
            .map_err(|e| Error::Transcription(format!("warmup inference failed: {e}")))?;

        Ok(())
    }

    pub fn transcribe(&self, audio: &[f32], language: Language) -> Result<String> {
        if audio.is_empty() {
            return Ok(String::new());
        }

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| Error::Transcription(format!("failed to create state: {e}")))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Set language
        let lang_key = match language {
            Language::English => "en",
            Language::German => "de",
        };
        params.set_language(Some(lang_key));

        // Suppress console output
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, audio)
            .map_err(|e| Error::Transcription(format!("transcription failed: {e}")))?;

        let num_segments = state.full_n_segments();

        let mut result = String::new();
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                let text = segment.to_str().map_err(|e| {
                    Error::Transcription(format!("failed to get segment text: {e}"))
                })?;
                result.push_str(text);
            }
        }

        Ok(result.trim().to_string())
    }
}
