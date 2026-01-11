# Scribe - Tauri Application
# Use `just --list` to see all available recipes

# Default recipe - show available commands
default:
    @just --list

# Install dependencies
install:
    npm install

# Run the application in development mode
dev:
    npm run tauri dev

# Build the application for release
build:
    npm run tauri build

# Build in debug mode (faster compilation)
build-debug:
    npm run tauri build -- --debug

# Check Rust code for errors without building
check:
    cargo check --manifest-path src-tauri/Cargo.toml

# Format Rust code
fmt:
    cargo fmt --manifest-path src-tauri/Cargo.toml

# Lint Rust code with clippy
lint:
    cargo clippy --manifest-path src-tauri/Cargo.toml

# Lint with stricter warnings
lint-pedantic:
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -W clippy::pedantic

# Run tests
test:
    cargo test --manifest-path src-tauri/Cargo.toml

# Run tests with output
test-verbose:
    cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture

# Clean build artifacts
clean:
    cargo clean --manifest-path src-tauri/Cargo.toml
    rm -rf node_modules dist

# Clean only Rust build artifacts
clean-rust:
    cargo clean --manifest-path src-tauri/Cargo.toml

# Update Rust dependencies
update:
    cargo update --manifest-path src-tauri/Cargo.toml

# Generate Tauri icons from a source image
icons source:
    npm run tauri icon {{source}}

# Open the bundle location after building
open-bundle:
    open src-tauri/target/release/bundle/macos/

# Build and open the bundle
release: build open-bundle

# Reset macOS permissions for dev builds
reset-permissions:
    sudo tccutil reset Accessibility com.scribe.app
    sudo tccutil reset AppleEvents com.scribe.app
