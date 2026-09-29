# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building and Running
```bash
# Development (using Nix flake - recommended)
nix develop      # Enter development shell
cargo run        # Build and run the application

# Manual build
cargo build --release   # Production build
./target/release/universal-search  # Run built binary

# Nix package build
nix build .      # Creates result/ symlink
./result/bin/universal-search  # Run packaged binary
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --lib                    # Unit tests only
cargo test --test integration_tests # Integration tests only
cargo test --test calculator_tests  # Calculator tests only

# Run tests with output
cargo test -- --nocapture

# Use test script (comprehensive)
./test.sh        # Runs tests, formatting, clippy, and release build
```

### Code Quality
```bash
cargo fmt        # Auto-format code
cargo fmt --check # Check formatting without changes
cargo clippy -- -D warnings  # Lint with warnings as errors
```

## Architecture Overview

### Core Components

**Main Application (`src/main.rs`)**
- `SearchWindow`: Main UI container using GTK4 for standard window functionality
- Window management: Standard desktop application window
- Entry widget for search input with real-time search triggering
- Results display as buttons in a vertical box with scrollable container

**Application Discovery (`src/apps.rs`)**
- `AppSearch`: Core search engine that interfaces with GIO to discover desktop applications
- `Application`: Wrapper around `DesktopAppInfo` with fuzzy matching scores
- Fuzzy search algorithm with configurable scoring (exact match > starts-with > contains > fuzzy)
- Frequency tracking for improving result relevance over time

**Mathematical Calculator (`src/calculator.rs`)**
- `calculate()`: Evaluates mathematical expressions using gnome-calculator
- `launch_calculator()`: Opens gnome-calculator with a given expression
- Smart math detection to distinguish between search queries and calculations
- Real-time calculation results displayed as "= result" format

**AI Assistant (`src/gemini.rs`)**
- `Gemini`: GObject-based service for AI search using Google's Gemini API
- Real-time AI responses with loading states and error handling
- Pango markup support for rich text formatting in responses
- Environment variable `PERSONAL_GEMINI_API_KEY` required for API access
- Automatic markup sanitization and validation for GTK4 display
- Thread-safe async implementation using glib's spawn_future_local
- Conversation history management (up to 5 exchanges)

**Styling (`src/style.css`)**
- GTK4 CSS using theme color variables (@theme_text_color, @wm_button_hover_color_a, etc.)
- Consistent styling with desktop theme
- Main container uses `.Centered` class for background/padding

### Key Design Decisions

**Application Behavior**:
- Unlimited results with scrollable display
- Empty query behavior (shows all available apps)
- Clean UI structure and styling

**Standard GTK Window**: Uses standard GTK4 ApplicationWindow:
- Resizable window with title bar
- Standard window controls and behavior
- Works on both X11 and Wayland

**Search Result Management**:
- All matching results displayed with scrollable container
- Empty query shows all installed apps
- Search query triggers fuzzy matching with score-based ranking
- ScrolledWindow with min/max content height for optimal display
- Click handlers for launching applications

### Testing Strategy

**Integration Tests (`tests/integration_tests.rs`)**:
- Tests actual GIO application discovery
- Validates fuzzy search scoring and limits
- Ensures application launching capability
- Tests frequency tracking functionality

**Library Structure (`src/lib.rs`)**:
- Exposes `apps`, `calculator`, `files`, `gemini`, and `history` modules publicly for testing
- Enables integration testing of internal components

**Example Tests**:
- `examples/gemini_test.rs`: Basic Gemini service functionality test

### Development Environment

**Nix Flake Setup**:
- Provides GTK4, layer-shell, and Rust toolchain
- Sets up proper environment variables for GTK schemas
- Includes development tools (rust-analyzer, clippy, rustfmt)

**Native Dependencies**: Requires GTK4, GLib/GIO for application discovery and UI functionality, gnome-calculator for mathematical evaluations, and internet connectivity for AI search functionality.

### Common Patterns

**Error Handling**: Uses `Result<(), glib::Error>` for application launching, with errors printed to stderr but not blocking UI operation.

**State Management**: Uses `Rc<RefCell<AppSearch>>` for shared access to search state across UI callbacks.

**UI Event Handling**: Button-specific click handling for app launching with keyboard shortcuts (Escape to close).