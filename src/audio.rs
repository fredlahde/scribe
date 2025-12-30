use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, Stream};
use rubato::{FftFixedIn, Resampler};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};

const WHISPER_SAMPLE_RATE: u32 = 16000;
const AUDIO_GAIN: f32 = 3.0; // Amplify audio by 3x

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    recording: Arc<AtomicBool>,
    _stream: Option<Stream>,
    sample_rate: u32,
    channels: usize,
}

impl AudioRecorder {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| Error::Audio("no input device available".to_string()))?;

        eprintln!("[Audio device: {:?}]", device.name());

        let config = device
            .default_input_config()
            .map_err(|e| Error::Audio(format!("failed to get default input config: {}", e)))?;

        eprintln!(
            "[Audio config: sample_rate={}, channels={}, format={:?}]",
            config.sample_rate().0,
            config.channels(),
            config.sample_format()
        );

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let recording = Arc::new(AtomicBool::new(false));

        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                build_input_stream::<f32>(&device, &config.into(), samples.clone(), recording.clone())
            }
            SampleFormat::I16 => {
                build_input_stream::<i16>(&device, &config.into(), samples.clone(), recording.clone())
            }
            SampleFormat::I32 => {
                build_input_stream::<i32>(&device, &config.into(), samples.clone(), recording.clone())
            }
            format => Err(Error::Audio(format!("unsupported sample format: {:?}", format))),
        }?;

        // Start the stream immediately and keep it running
        stream
            .play()
            .map_err(|e| Error::Audio(format!("failed to start stream: {}", e)))?;

        Ok(Self {
            samples,
            recording,
            _stream: Some(stream),
            sample_rate,
            channels,
        })
    }

    pub fn start(&self) -> Result<()> {
        // Clear any previous samples and start recording
        self.samples.lock().unwrap().clear();
        self.recording.store(true, Ordering::SeqCst);
        eprintln!("[Recording started]");
        Ok(())
    }

    pub fn stop(&self) -> Result<Vec<f32>> {
        // Keep recording flag on for a moment to capture trailing audio
        std::thread::sleep(std::time::Duration::from_millis(150));
        self.recording.store(false, Ordering::SeqCst);

        let raw_samples = self.samples.lock().unwrap().clone();
        eprintln!("[Raw samples collected: {}]", raw_samples.len());

        if raw_samples.is_empty() {
            return Ok(Vec::new());
        }

        // Convert stereo to mono if needed
        let mono = if self.channels > 1 {
            stereo_to_mono(&raw_samples, self.channels)
        } else {
            raw_samples
        };

        // Resample to 16kHz if needed
        if self.sample_rate != WHISPER_SAMPLE_RATE {
            resample(&mono, self.sample_rate, WHISPER_SAMPLE_RATE)
        } else {
            Ok(mono)
        }
    }
}

fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    recording: Arc<AtomicBool>,
) -> Result<Stream>
where
    T: cpal::Sample + cpal::SizedSample,
    f32: FromSample<T>,
{
    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                if recording.load(Ordering::SeqCst) {
                    let mut buffer = samples.lock().unwrap();
                    for &sample in data {
                        // Apply gain and clamp to prevent clipping
                        let amplified = (f32::from_sample(sample) * AUDIO_GAIN).clamp(-1.0, 1.0);
                        buffer.push(amplified);
                    }
                }
            },
            |err| eprintln!("audio stream error: {}", err),
            None,
        )
        .map_err(|e| Error::Audio(format!("failed to build input stream: {}", e)))?;

    Ok(stream)
}

fn stereo_to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    samples
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>> {
    // Use a reasonable chunk size for the resampler
    let chunk_size = 1024;
    
    let mut resampler = FftFixedIn::<f32>::new(
        from_rate as usize,
        to_rate as usize,
        chunk_size,
        1, // sub_chunks
        1, // channels (mono)
    )
    .map_err(|e| Error::Resample(format!("failed to create resampler: {}", e)))?;

    let mut output = Vec::new();
    
    // Process in chunks
    for chunk in samples.chunks(chunk_size) {
        // Pad the last chunk if needed
        let input_chunk = if chunk.len() < chunk_size {
            let mut padded = chunk.to_vec();
            padded.resize(chunk_size, 0.0);
            padded
        } else {
            chunk.to_vec()
        };
        
        let waves_in = vec![input_chunk];
        let waves_out = resampler
            .process(&waves_in, None)
            .map_err(|e| Error::Resample(format!("failed to resample: {}", e)))?;
        
        if let Some(channel) = waves_out.into_iter().next() {
            output.extend(channel);
        }
    }

    eprintln!("[Resampled {} -> {} samples]", samples.len(), output.len());
    Ok(output)
}
