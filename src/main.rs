mod audio;
mod error;
mod input;
mod transcribe;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::thread;

use rdev::{Event, EventType, Key, listen};

use crate::audio::AudioRecorder;
use crate::error::Result;
use crate::input::TextInput;
use crate::transcribe::Transcriber;

const MODEL_PATH: &str = "./ggml-medium.bin";

#[derive(Debug)]
enum HotkeyEvent {
    StartRecording,
    StopRecordingEnglish,
    StopRecordingGerman,
}

fn main() -> Result<()> {
    println!("Loading whisper model...");
    let transcriber = Transcriber::new(MODEL_PATH)?;
    println!("Model loaded.");

    let mut text_input = TextInput::new()?;
    let recorder = AudioRecorder::new()?;

    println!("Registered hotkey: F2");
    println!("Hold F2 to record, release to transcribe and type.");
    println!("Press Ctrl+C to exit.");

    // Channel to send hotkey events from listener thread to main thread
    let (tx, rx): (_, Receiver<HotkeyEvent>) = channel();

    // Spawn keyboard listener on a separate thread
    thread::spawn(move || {
        let recording = Arc::new(AtomicBool::new(false));

        let callback = move |event: Event| match event.event_type {
            EventType::KeyPress(Key::F2) | EventType::KeyPress(Key::F3) => {
                if !recording.load(Ordering::SeqCst) {
                    recording.store(true, Ordering::SeqCst);
                    let _ = tx.send(HotkeyEvent::StartRecording);
                }
            }
            EventType::KeyRelease(Key::F2) => {
                if recording.load(Ordering::SeqCst) {
                    recording.store(false, Ordering::SeqCst);
                    let _ = tx.send(HotkeyEvent::StopRecordingEnglish);
                }
            }
            EventType::KeyRelease(Key::F3) => {
                if recording.load(Ordering::SeqCst) {
                    recording.store(false, Ordering::SeqCst);
                    let _ = tx.send(HotkeyEvent::StopRecordingGerman);
                }
            }
            _ => {}
        };

        if let Err(e) = listen(callback) {
            eprintln!("Keyboard listener error: {:?}", e);
        }
    });

    // Main loop processes hotkey events and handles audio
    loop {
        match rx.recv() {
            Ok(HotkeyEvent::StartRecording) => {
                if let Err(e) = recorder.start() {
                    eprintln!("[Recording error: {}]", e);
                    continue;
                }
                eprintln!("[Recording...]");
            }
            Ok(HotkeyEvent::StopRecordingEnglish) => {
                let audio = match recorder.stop() {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("[Stop error: {}]", e);
                        continue;
                    }
                };
                eprintln!("[Transcribing {} samples in english...]", audio.len());

                if !audio.is_empty() {
                    match transcriber.transcribe(&audio, transcribe::Language::English) {
                        Ok(text) => {
                            if !text.is_empty() {
                                eprintln!("[Typing: {}]", text);
                                if let Err(e) = text_input.type_text(&text) {
                                    eprintln!("[Type error: {}]", e);
                                }
                            } else {
                                eprintln!("[No speech detected]");
                            }
                        }
                        Err(e) => {
                            eprintln!("[Transcription error: {}]", e);
                        }
                    }
                } else {
                    eprintln!("[No audio captured]");
                }
            }
            Ok(HotkeyEvent::StopRecordingGerman) => {
                let audio = match recorder.stop() {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("[Stop error: {}]", e);
                        continue;
                    }
                };
                eprintln!("[Transcribing {} samples in german...]", audio.len());

                if !audio.is_empty() {
                    match transcriber.transcribe(&audio, transcribe::Language::German) {
                        Ok(text) => {
                            if !text.is_empty() {
                                eprintln!("[Typing: {}]", text);
                                if let Err(e) = text_input.type_text(&text) {
                                    eprintln!("[Type error: {}]", e);
                                }
                            } else {
                                eprintln!("[No speech detected]");
                            }
                        }
                        Err(e) => {
                            eprintln!("[Transcription error: {}]", e);
                        }
                    }
                } else {
                    eprintln!("[No audio captured]");
                }
            }
            Err(_) => {
                // Channel closed, listener thread died
                break;
            }
        }
    }

    Ok(())
}
