# AGENTS.md - AI Coding Agent Guidelines

This document provides guidelines for AI coding agents working in the `whisper_to_me` codebase.

## Project Overview

A Rust push-to-talk voice transcription application for macOS. Captures audio when the user holds Cmd+Shift+Space, transcribes speech using OpenAI's Whisper model (via whisper-rs with CoreML acceleration), and automatically types the transcribed text.

### Architecture

| Module | Purpose |
|--------|---------|
| `main.rs` | Entry point, hotkey listener, orchestrates recording/transcription pipeline |
| `audio.rs` | Microphone capture via cpal, stereo-to-mono conversion, resampling to 16kHz |
| `transcribe.rs` | Whisper model loading and speech-to-text conversion |
| `error.rs` | Centralized error handling with `thiserror` |
| `input.rs` | Text input simulation using `enigo` |

---

## Build Commands

```bash
# Build (debug)
cargo build

# Build (release, recommended for performance)
cargo build --release

# Run
cargo run --release

# Check for errors without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy

# Lint with stricter warnings
cargo clippy -- -W clippy::pedantic
```

## Testing

```bash
# Run all tests
cargo test

# Run a single test by name
cargo test test_name

# Run tests in a specific module
cargo test module_name::

# Run tests with output
cargo test -- --nocapture

# Run tests matching a pattern
cargo test pattern
```

**Note**: This codebase currently has no tests. When adding tests, place them in a `#[cfg(test)]` module at the bottom of each source file.

---

## Code Style Guidelines

### Module Organization

- Flat module structure with `mod` declarations at top of `main.rs`
- Module order: alphabetical (`audio`, `error`, `input`, `transcribe`)
- One module per file under `src/`

### Import Style

Order imports as follows (with blank lines between groups):

```rust
// 1. External crates
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use thiserror::Error;

// 2. Standard library
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// 3. Crate-local imports
use crate::error::{Error, Result};
```

- Use explicit paths: `use crate::module::Type`
- Group related imports with braces: `use std::sync::{Arc, Mutex};`
- Import traits explicitly when using their methods

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Types/Structs/Enums | PascalCase | `AudioRecorder`, `HotkeyEvent` |
| Functions/Methods | snake_case | `stereo_to_mono`, `build_input_stream` |
| Constants | SCREAMING_SNAKE_CASE | `WHISPER_SAMPLE_RATE`, `MODEL_PATH` |
| Variables | snake_case | `raw_samples`, `sample_rate` |
| Modules | snake_case | `transcribe`, `audio` |

### Error Handling

Use the centralized error system in `error.rs`:

```rust
use crate::error::{Error, Result};

// Return Result<T> from fallible functions
pub fn do_something() -> Result<SomeType> {
    // Use .map_err() to convert external errors
    some_fallible_call()
        .map_err(|e| Error::Audio(format!("description: {}", e)))?;
    
    Ok(result)
}
```

**Error message style**:
- Lowercase first letter
- Descriptive and specific
- Include context: `"failed to get default input config: {}"`

**Error variants** (in `error.rs`):
- `Audio(String)` - Audio capture/processing errors
- `Transcription(String)` - Whisper model errors
- `Input(enigo::InputError)` - Text input errors (uses `#[from]`)
- `Hotkey(String)` - Keyboard listener errors
- `Resample(String)` - Audio resampling errors

### Struct Patterns

```rust
pub struct SomeThing {
    field: Type,  // Private fields by default
}

impl SomeThing {
    // Constructor returns Result
    pub fn new(param: Type) -> Result<Self> {
        Ok(Self { field: value })
    }
    
    // Methods take &self or &mut self
    pub fn do_action(&self) -> Result<Output> {
        // implementation
    }
}
```

### Concurrency Patterns

This codebase uses synchronous threading (not async):

```rust
// Thread-safe shared state
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

// Shared mutable data
let buffer = Arc::new(Mutex::new(Vec::new()));

// Atomic flags
let is_active = Arc::new(AtomicBool::new(false));
is_active.store(true, Ordering::SeqCst);

// Cross-thread communication
use std::sync::mpsc::{channel, Receiver, Sender};
let (tx, rx) = channel();
```

### Formatting

- Use `cargo fmt` (rustfmt defaults)
- 4-space indentation
- Trailing commas in multi-line constructs
- Max line width: 100 characters (rustfmt default)

### Comments and Logging

```rust
// Single-line comments for explanations

// Debug output uses eprintln! with bracketed prefixes
eprintln!("[Recording...]");
eprintln!("[Transcribing...]");
```

### Generic Type Bounds

Use where clauses for complex bounds:

```rust
fn process_samples<T>(samples: &[T]) -> Vec<f32>
where
    T: cpal::Sample + cpal::SizedSample,
    f32: FromSample<T>,
{
    // implementation
}
```

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `whisper-rs` | 0.15.1 | Whisper bindings (CoreML enabled) |
| `cpal` | 0.15 | Cross-platform audio I/O |
| `rdev` | 0.5 | Global keyboard event listener |
| `enigo` | 0.6 | Cross-platform input simulation |
| `rubato` | 0.15 | High-quality audio resampling |
| `thiserror` | 2.0 | Error derive macro |

---

## Platform Notes

- **Target**: macOS (uses CoreML for Whisper acceleration, Meta key for hotkey)
- **Model file**: Requires `ggml-medium.bin` in project root (gitignored)
- **Hotkey**: Cmd+Shift+Space (hardcoded in `main.rs`)
- **Audio format**: 16kHz mono (Whisper requirement)

---

## Common Tasks

### Adding a New Error Variant

1. Add variant to `Error` enum in `src/error.rs`
2. Use `#[error("message")]` for the display format
3. Use `#[from]` if auto-conversion from another error type is needed

### Adding a New Module

1. Create `src/module_name.rs`
2. Add `mod module_name;` to `main.rs` (maintain alphabetical order)
3. Add public exports as needed
