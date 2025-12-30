# Whisper to Me

A macOS push-to-talk voice transcription app built with Tauri. Hold a hotkey to record, release to transcribe using OpenAI's Whisper model with CoreML/Metal acceleration, and automatically type the result.

## Features

- **Push-to-talk recording** - Hold a configurable hotkey to record, release to transcribe
- **Fast local transcription** - Uses Whisper.cpp with CoreML and Metal GPU acceleration
- **Multi-language support** - Separate hotkeys for English and German transcription
- **Auto-typing** - Transcribed text is automatically typed at your cursor position
- **System tray app** - Lives in your menu bar with status indicators
- **Mute toggle** - Quickly disable/enable the microphone with a hotkey
- **Configurable hotkeys** - Set custom keyboard shortcuts for each action

## Requirements

- macOS 10.15 or later
- Microphone access permission
- Accessibility permission (for auto-typing)
- A Whisper model file (see below)

## Model Setup

You need to download a Whisper model to use this app. The recommended model is `ggml-medium.bin` with its CoreML encoder for optimal performance on Apple Silicon.

### Download the Model Files

1. **Whisper model** (required):
   - [ggml-medium.bin](https://huggingface.co/ggerganov/whisper.cpp/blob/main/ggml-medium.bin)

2. **CoreML encoder** (recommended for Apple Silicon):
   - [ggml-medium-encoder.mlmodelc.zip](https://huggingface.co/ggerganov/whisper.cpp/blob/main/ggml-medium-encoder.mlmodelc.zip)

### Install the Model

1. Download `ggml-medium.bin` and place it in a convenient location (e.g., `~/Models/`)
2. Download and unzip `ggml-medium-encoder.mlmodelc.zip`
3. Place the unzipped `ggml-medium-encoder.mlmodelc` folder in the **same directory** as the `.bin` file
4. On first launch, configure the model path in the settings window

## Installation

### From Release

Download the latest `.dmg` from the [Releases](../../releases) page and drag the app to your Applications folder.

### Build from Source

#### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [just](https://github.com/casey/just) command runner (optional but recommended)

#### Build Steps

```bash
# Install dependencies
npm install

# Development mode
just dev
# or: npm run tauri dev

# Production build
just build
# or: npm run tauri build
```

The built app will be in `src-tauri/target/release/bundle/macos/`.

## Usage

### First Launch

1. Launch the app - the settings window will open automatically if no model is configured
2. Click **Browse** and select your `ggml-medium.bin` file
3. Configure your preferred hotkeys (defaults: F2 for English, F4 for mute)
4. Click **Save**
5. Grant microphone and accessibility permissions when prompted

### Recording

1. Position your cursor where you want the text to appear
2. **Hold** the English hotkey (default: F2) and speak
3. **Release** the hotkey when done - the app will transcribe and type the result

### System Tray

The app lives in your menu bar with status icons:

- **Idle** - Ready to record
- **Recording** - Currently capturing audio
- **Transcribing** - Processing speech to text
- **Muted** - Microphone disabled

Right-click the tray icon for options (Settings, Quit).

### Hotkeys

| Default   | Action                           |
| --------- | -------------------------------- |
| F2 (hold) | Record in English                |
| F3 (hold) | Record in German (if configured) |
| F4        | Toggle mute                      |

All hotkeys can be customized in Settings.

## Development

```bash
just dev          # Run in development mode
just check        # Check for compile errors (fast)
just fmt          # Format code
just lint         # Run clippy linter
just test         # Run tests
```

See `just --list` for all available commands.

## Project Structure

```
whisper_to_me/
├── src/                    # Frontend (vanilla JS)
│   ├── main.js            # Settings UI logic
│   └── styles.css         # Styles
├── src-tauri/             # Rust backend
│   └── src/
│       ├── lib.rs         # App setup, shortcuts, orchestration
│       ├── audio.rs       # Microphone capture, resampling
│       ├── transcribe.rs  # Whisper integration
│       ├── input.rs       # Text typing simulation
│       ├── settings.rs    # App state management
│       ├── tray.rs        # System tray handling
│       └── error.rs       # Error types
├── index.html             # Settings window
└── justfile               # Build commands
```

## Troubleshooting

### "Microphone access denied"

Grant microphone permission in System Preferences > Security & Privacy > Privacy > Microphone.

### Text not typing

Grant accessibility permission in System Preferences > Security & Privacy > Privacy > Accessibility.

### Slow transcription

Ensure the CoreML encoder (`ggml-medium-encoder.mlmodelc`) is in the same directory as your model file. This enables GPU acceleration on Apple Silicon.

### Model fails to load

Verify the model file path is correct and the file is not corrupted. Try re-downloading the model.

## License

MIT
