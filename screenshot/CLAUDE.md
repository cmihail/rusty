# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a GTK4-based screenshot application written in Rust that provides a GUI for taking screenshots on Wayland compositors. It's inspired by an original AGS TypeScript implementation and uses external tools like `grim` and `slurp` for actual screenshot capture.

## Architecture

The application has a clean separation between UI (`main.rs`) and screenshot logic (`screenshot.rs`):

- **main.rs**: GTK4 UI implementation with a centered dialog containing screenshot option buttons
- **screenshot.rs**: Core screenshot functionality including command building, execution, and notification handling
- **lib.rs**: Library interface for exposing screenshot functionality to integration tests

The app creates commands that shell out to external Wayland screenshot tools rather than implementing screenshot capture directly.

## Development Commands

### Building and Running
```bash
# Build the application
cargo build

# Build for release
cargo build --release

# Run the application
cargo run

# Using Nix (preferred development environment)
nix develop
cargo run
```

### Testing
```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run tests with output
cargo test -- --nocapture

# Run the comprehensive test script (includes linting)
./test.sh
```

### Code Quality
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run lints
cargo clippy -- -D warnings
```

## Dependencies

### Runtime Dependencies (External Tools)
- `grim` - Wayland screenshot utility
- `slurp` - Screen area selection tool
- `hyprctl` - Hyprland control utility (optional, for monitor/window detection)
- `notify-send` - Desktop notifications

### Rust Dependencies
- `gtk4` - GUI framework
- `chrono` - Timestamp generation
- `tokio` - Async runtime (for process execution)

## Key Implementation Details

### Screenshot Command Building
Commands are built as shell strings with proper quoting to prevent injection attacks. All file paths are wrapped in single quotes for safety.

### Screenshot Types
- **All monitors**: `grim 'output_file'`
- **Select area**: `grim -g "$(slurp)" 'output_file'`
- **Focused monitor**: Currently falls back to all monitors
- **Focused window**: Currently falls back to select area

### File Organization
Screenshots are saved to `~/Pictures/Screenshots/` with timestamp filenames in format `YYYY.MM.DD-HH:MM:SS.png`.

### Testing Strategy
- Unit tests in `src/screenshot.rs` test command building, path safety, and error handling
- Integration tests in `tests/integration_tests.rs` test environment variable handling and cross-platform behavior
- The `test.sh` script runs comprehensive testing including linting

## Development Environment

The project supports both traditional Rust development and Nix-based development:

- **Nix flake** provides complete development environment with all dependencies
- **Native development** requires manual installation of GTK4 development packages and screenshot tools

When developing, the application expects to find `src/style.css` for GTK4 styling.