# AGENTS.md - AI Coding Agent Guidelines

## Project Overview

A Tauri-based push-to-talk voice transcription app for macOS. Captures audio via configurable hotkeys, transcribes using Whisper (with CoreML/Metal acceleration), and types the result.

### Architecture

| Module | Purpose |
|--------|---------|
| `lib.rs` | Tauri app setup, global shortcuts, recording/transcription orchestration |
| `audio.rs` | Microphone capture via cpal, stereo-to-mono conversion, resampling to 16kHz |
| `transcribe.rs` | Whisper model loading and speech-to-text (supports English/German) |
| `error.rs` | Centralized error handling with `thiserror` |
| `input.rs` | Text input simulation using `enigo` (lazy initialization) |
| `settings.rs` | Thread-safe application state management |
| `tray.rs` | System tray icon and menu handling |
| `src/main.js` | Frontend settings UI (vanilla JS) |

---

## Build Commands

```bash
just dev                    # Development mode (or: npm run tauri dev)
just build                  # Production build (or: npm run tauri build)
just check                  # Check Rust code for errors (fast)
just fmt                    # Format code
just lint                   # Lint with clippy
just lint-pedantic          # Lint with stricter warnings
```

## Testing

```bash
just test                   # Run all tests
just test-verbose           # Run tests with output

# Run a single test by name
cargo test test_name --manifest-path src-tauri/Cargo.toml

# Run tests in a specific module
cargo test module_name:: --manifest-path src-tauri/Cargo.toml
```

**Note**: Tests should be placed in `#[cfg(test)]` modules at the bottom of each source file.

---

## Code Style Guidelines

### Module Organization

- Modules declared alphabetically in `lib.rs`: `audio`, `error`, `input`, `settings`, `transcribe`, `tray`
- One module per file under `src-tauri/src/`

### Import Style

```rust
// 1. External crates
use tauri::Manager;

// 2. Standard library
use std::sync::{Arc, Mutex};

// 3. Crate-local imports
use crate::error::{Error, Result};
```

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Types/Structs/Enums | PascalCase | `AudioRecorder`, `RecordingState` |
| Functions/Methods | snake_case | `stereo_to_mono`, `handle_recording_stop` |
| Constants | SCREAMING_SNAKE_CASE | `WHISPER_SAMPLE_RATE`, `TRAY_ID` |

### Error Handling

Use the centralized error system in `error.rs`:

```rust
use crate::error::{Error, Result};

pub fn do_something() -> Result<SomeType> {
    fallible_call()
        .map_err(|e| Error::Audio(format!("description: {}", e)))?;
    Ok(result)
}
```

**Error variants**: `Audio`, `Transcription`, `Input`, `InputCreation`, `Hotkey`, `Resample`

**Style**: lowercase first letter, descriptive, include context.

### Concurrency Patterns

Synchronous threading (not async for core logic):

```rust
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

let buffer = Arc::new(Mutex::new(Vec::new()));
let recording = Arc::new(AtomicBool::new(false));
recording.store(true, Ordering::SeqCst);
```

### Tauri Commands

```rust
#[tauri::command]
async fn my_command(app: tauri::AppHandle) -> Result<(), String> {
    do_something().map_err(|e| format!("Failed: {}", e))?;
    Ok(())
}
```

### Formatting & Logging

- Use `cargo fmt` (rustfmt defaults), 4-space indentation, trailing commas
- Debug output: `eprintln!("[Recording started]");`

---

## Platform Notes

- **Target**: macOS (CoreML/Metal for Whisper, system tray)
- **Model**: User-configured `.bin` file path stored in settings
- **Hotkeys**: Configurable via settings UI (default: F2 for English)
- **Audio format**: 16kHz mono (Whisper requirement)
- **Permissions**: Requires microphone + accessibility access

---

## Common Tasks

### Adding a New Error Variant

1. Add variant to `Error` enum in `src-tauri/src/error.rs`
2. Use `#[error("message")]` for display format
3. Use `#[from]` for auto-conversion from external error types

### Adding a Tauri Command

1. Add `#[tauri::command]` function in `lib.rs`
2. Register in `.invoke_handler(tauri::generate_handler![...])`
3. Call from frontend: `await invoke("command_name", { args })`

### Adding a New Language

1. Add variant to `Language` enum in `transcribe.rs`
2. Add language code mapping in `transcribe()` method
3. Add hotkey setting in frontend (`main.js`) and backend (`lib.rs`)
