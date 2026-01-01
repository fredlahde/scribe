use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Sample, SampleFormat, Stream, StreamConfig};
use rubato::{FftFixedIn, Resampler};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};

const WHISPER_SAMPLE_RATE: u32 = 16000;

/// Helper to get device name from description
fn get_device_name(device: &Device) -> String {
    device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

/// Returns a list of available audio input device names.
pub fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    match host.input_devices() {
        Ok(devices) => devices.map(|d| get_device_name(&d)).collect(),
        Err(e) => {
            eprintln!("[Failed to enumerate input devices: {}]", e);
            Vec::new()
        }
    }
}

/// Checks if an audio device with the given name exists.
/// Returns true if device_name is None (system default) or if the device is found.
pub fn device_exists(device_name: Option<&str>) -> bool {
    let Some(name) = device_name else {
        return true; // None means system default, always valid
    };

    if name.is_empty() {
        return true; // Empty string treated as system default
    }

    let host = cpal::default_host();
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if get_device_name(&device) == name {
                return true;
            }
        }
    }
    false
}

/// Finds an input device by name, falling back to default if not found.
fn find_device_by_name(device_name: Option<&str>) -> Result<Device> {
    let host = cpal::default_host();

    match device_name {
        // Treat empty string like `None` (system default), consistent with `device_exists`
        Some(name) if name.is_empty() => {
            return host
                .default_input_device()
                .ok_or_else(|| Error::Audio("no input device available".to_string()));
        }
        Some(name) => {
            if let Ok(devices) = host.input_devices() {
                for device in devices {
                    let device_name_str = get_device_name(&device);
                    if device_name_str == name {
                        eprintln!("[Using audio device: {}]", name);
                        return Ok(device);
                    }
                }
            }
            eprintln!("[Device '{}' not found, falling back to default]", name);
        }
        None => {
            // No specific device requested; fall through to default below.
        }
    }

    host.default_input_device()
        .ok_or_else(|| Error::Audio("no input device available".to_string()))
}
const AUDIO_GAIN: f32 = 3.0; // Amplify audio by 3x

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    recording: Arc<AtomicBool>,
    stream: Option<Stream>,
    sample_rate: u32,
    channels: usize,
    // Store device and config for recreating stream after unmute
    device: Device,
    stream_config: StreamConfig,
    sample_format: SampleFormat,
}

impl AudioRecorder {
    pub fn new(device_name: Option<&str>) -> Result<Self> {
        let device = find_device_by_name(device_name)?;

        eprintln!("[Audio device: {}]", get_device_name(&device));

        let config = device
            .default_input_config()
            .map_err(|e| Error::Audio(format!("failed to get default input config: {}", e)))?;

        eprintln!(
            "[Audio config: sample_rate={}, channels={}, format={:?}]",
            config.sample_rate(),
            config.channels(),
            config.sample_format()
        );

        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.clone().into();
        let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let recording = Arc::new(AtomicBool::new(false));

        let stream = Self::create_stream(
            &device,
            &stream_config,
            sample_format,
            samples.clone(),
            recording.clone(),
        )?;

        // Start the stream immediately and keep it running
        stream
            .play()
            .map_err(|e| Error::Audio(format!("failed to start stream: {}", e)))?;

        Ok(Self {
            samples,
            recording,
            stream: Some(stream),
            sample_rate,
            channels,
            device,
            stream_config,
            sample_format,
        })
    }

    fn create_stream(
        device: &Device,
        config: &StreamConfig,
        sample_format: SampleFormat,
        samples: Arc<Mutex<Vec<f32>>>,
        recording: Arc<AtomicBool>,
    ) -> Result<Stream> {
        let stream = match sample_format {
            SampleFormat::F32 => build_input_stream::<f32>(device, config, samples, recording),
            SampleFormat::I16 => build_input_stream::<i16>(device, config, samples, recording),
            SampleFormat::I32 => build_input_stream::<i32>(device, config, samples, recording),
            format => Err(Error::Audio(format!(
                "unsupported sample format: {:?}",
                format
            ))),
        }?;

        Ok(stream)
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

    /// Mute the microphone by stopping and dropping the audio stream.
    /// This releases the microphone so the system no longer shows it as in use.
    pub fn mute(&mut self) -> Result<()> {
        if self.stream.is_none() {
            // Already muted
            return Ok(());
        }

        // Drop the stream to release the microphone
        self.stream = None;
        self.recording.store(false, Ordering::SeqCst);
        eprintln!("[Microphone muted]");
        Ok(())
    }

    /// Unmute the microphone by recreating and starting the audio stream.
    pub fn unmute(&mut self) -> Result<()> {
        if self.stream.is_some() {
            // Already unmuted
            return Ok(());
        }

        let stream = Self::create_stream(
            &self.device,
            &self.stream_config,
            self.sample_format,
            self.samples.clone(),
            self.recording.clone(),
        )?;

        stream
            .play()
            .map_err(|e| Error::Audio(format!("failed to start stream: {}", e)))?;

        self.stream = Some(stream);
        eprintln!("[Microphone unmuted]");
        Ok(())
    }

    /// Check if the microphone is currently muted.
    pub fn is_muted(&self) -> bool {
        self.stream.is_none()
    }

    /// Switch to a different audio input device.
    /// If device_name is None or the device is not found, falls back to the default device.
    pub fn set_device(&mut self, device_name: Option<&str>) -> Result<()> {
        // Find the new device
        let device = find_device_by_name(device_name)?;
        let device_name_str = get_device_name(&device);

        // Get the new device's config
        let config = device
            .default_input_config()
            .map_err(|e| Error::Audio(format!("failed to get default input config: {}", e)))?;

        eprintln!(
            "[Switching to device: {} (sample_rate={}, channels={}, format={:?})]",
            device_name_str,
            config.sample_rate(),
            config.channels(),
            config.sample_format()
        );

        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();
        let stream_config: StreamConfig = config.into();

        // Stop current stream if running
        let was_muted = self.stream.is_none();
        self.stream = None;
        self.recording.store(false, Ordering::SeqCst);
        self.samples.lock().unwrap().clear();

        // Update device info
        self.device = device;
        self.stream_config = stream_config;
        self.sample_format = sample_format;
        self.sample_rate = sample_rate;
        self.channels = channels;

        // Recreate stream if we weren't muted
        if !was_muted {
            let stream = Self::create_stream(
                &self.device,
                &self.stream_config,
                self.sample_format,
                self.samples.clone(),
                self.recording.clone(),
            )?;

            stream
                .play()
                .map_err(|e| Error::Audio(format!("failed to start stream: {}", e)))?;

            self.stream = Some(stream);
        }

        eprintln!("[Audio device switched to: {}]", device_name_str);
        Ok(())
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
