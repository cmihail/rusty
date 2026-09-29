# Universal Search App

A GTK4-based universal search application written in Rust. Provides a desktop application launcher with fuzzy search capabilities.

## Features

- **App Search**: Fuzzy search through installed applications
- **Mathematical Calculator**: Perform calculations with real-time results
- **Quick Launch**: Launch applications directly from search results
- **Scrollable Results**: View all matching applications with smooth scrolling
- **Frequency Tracking**: Learns from usage patterns for better results
- **Keyboard Navigation**: Full keyboard support with shortcuts

## Dependencies

The application requires:
- GTK4 for the UI
- GIO/GLib for application discovery
- Desktop environment with .desktop files
- gnome-calculator for mathematical evaluations

## Building and Running

### Using Nix Flakes (Recommended)

**Installation:**

```bash
# Build the package (creates result symlink)
nix build .

# Then run the built binary
./result/bin/universal-search
```

**Development:**

```bash
# Enter development shell
nix develop

# Build and run
cargo run
```

### Using Fedora Packages

First, install the required packages:

```bash
# Install Rust toolchain
sudo dnf install rust cargo clippy

# Install GTK4 development packages
sudo dnf install gtk4-devel gtk4-layer-shell-devel glib2-devel gdk-pixbuf2-devel pango-devel cairo-devel atk-devel pkg-config

```

Then build and run:

```bash
# Build
cargo build --release

# Run
cargo run
```

## Testing

The project includes tests for the core functionality.

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Format and lint
cargo fmt
cargo clippy -- -D warnings
```

### Test Coverage

The test suite includes:

- **Unit tests** (`src/apps.rs`):
  - Application discovery and parsing
  - Fuzzy search algorithm
  - Scoring and ranking logic
  - Frequency tracking

- **Calculator tests** (`tests/calculator_tests.rs`):
  - Basic arithmetic operations
  - Complex mathematical expressions
  - Error handling and edge cases
  - Integration with search functionality

## Usage

1. Run the application
2. A window will appear with a search interface
3. Start typing to search for:
   - **Applications**: Type app names for fuzzy search
   - **Mathematics**: Type calculations like `2+2`, `(5+3)*2`, `sqrt(16)`
4. Click on results to:
   - **Apps**: Launch the application
   - **Math**: Open calculator with your expression
5. Press Escape to close the window

## Keyboard Shortcuts

- **Escape**: Close the search window
- **Enter**: Launch selected application or perform web search
- **Ctrl+C**: Clear search text
- Click outside the dialog to close it

## File Structure

```
universal-search/
├── Cargo.toml          # Rust dependencies and project config
├── src/
│   ├── main.rs         # Main application and UI code
│   ├── apps.rs         # Application search logic
│   └── style.css       # GTK4 CSS styling
├── flake.nix           # Nix flake for development environment
└── README.md           # This file
```

## Installation

To install the app system-wide, you can create a symlink to your desired location:

```bash
nix build .
ln -s $(pwd)/result/bin/universal-search <install_location>
```

For example, to install to `~/.local/bin`:
```bash
ln -s $(pwd)/result/bin/universal-search ~/.local/bin/universal-search
```

OR run:
```
nix profile install .
nix profile upgrade universal-search # if app already installed
```

## Configuration

The application automatically discovers applications from:
- System application directories (`/usr/share/applications/`)
- User application directories (`~/.local/share/applications/`)
- Flatpak applications (if available)
- Snap applications (if available)

### API Keys

Web search (Brave Search) and AI search (Gemini) read their keys from environment variables:

- `BRAVE_SEARCH_API_KEY`
- `PERSONAL_GEMINI_API_KEY`

To avoid storing them in plain text, keep them in the desktop keyring (Secret Service) with `secret-tool`:

```bash
# Install secret-tool
sudo apt install libsecret-tools   # Ubuntu/Debian
sudo dnf install libsecret         # Fedora

# Store the keys (prompts for the value)
secret-tool store --label="Brave Search API" service rusty key brave
secret-tool store --label="Gemini API" service rusty key gemini
```

Then launch the app through a wrapper that exports the keys:

```sh
#!/bin/sh
export BRAVE_SEARCH_API_KEY="$(secret-tool lookup service rusty key brave)"
export PERSONAL_GEMINI_API_KEY="$(secret-tool lookup service rusty key gemini)"
exec "$HOME/.nix-profile/bin/rusty-universal-search" "$@"
```

## Future Features

The current implementation focuses on app search. Future expansions could include:

- **File Search**: Search through files and directories
- **Calculator**: Inline mathematical calculations
- **Web Search**: Integration with search engines
- **History**: Search through command/application history
- **Plugins**: Extensible search providers

## Notes

- The application works on both X11 and Wayland
- Application frequency tracking helps improve search result relevance over time
- All matching applications are displayed with scrollable results for easy browsing