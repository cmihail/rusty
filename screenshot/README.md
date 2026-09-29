# Screenshot App

A GTK4-based screenshot application written in Rust, inspired by the original AGS TypeScript implementation.

## Features

- **All monitors**: Capture all connected monitors
- **Focused monitor**: Capture the currently focused monitor (requires Hyprland)
- **Focused window**: Capture the currently focused window (requires Hyprland)
- **Select area**: Interactive area selection using slurp

## Dependencies

The application requires the following tools to be installed:

- `grim` - Screenshot utility for Wayland
- `slurp` - Screen area selection tool for Wayland
- `hyprctl` - Hyprland control utility (optional, for monitor/window detection)

## Building and Running

### Using Nix Flakes (Recommended)

**Installation:**

```bash
# Build the package (creates result symlink)
nix build .

# Then run the built binary
./result/bin/screenshot
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
sudo dnf install rust cargo

# Install GTK4 development packages
sudo dnf install gtk4-devel glib2-devel gdk-pixbuf2-devel pango-devel cairo-devel atk-devel pkg-config

# Install screenshot tools
sudo dnf install grim slurp

# Install notification support
sudo dnf install libnotify

# For Hyprland users (if using Hyprland)
sudo dnf install hyprland
```

Then build and run:

```bash
# Build
cargo build --release

# Run
cargo run
```

## Testing

The project includes comprehensive tests to ensure reliability and correctness.

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test integration_tests

# Run tests with output
cargo test -- --nocapture

# Use the test script (includes linting and formatting checks)
./test.sh
```

### Test Coverage

The test suite includes:

- **Unit tests** (`src/screenshot.rs`):
  - File path generation and formatting
  - Command building and safety
  - Special character handling
  - Error handling
  - Timestamp validation

- **Integration tests** (`tests/integration_tests.rs`):
  - Environment variable handling
  - Directory structure validation
  - Command injection prevention
  - Cross-platform path handling

### Test Categories

1. **Command Building Tests**: Verify that screenshot commands are properly constructed
2. **Path Safety Tests**: Ensure file paths are properly escaped and safe
3. **Error Handling Tests**: Test error conditions and failure scenarios
4. **Format Validation Tests**: Verify output file naming and structure
5. **Environment Tests**: Test behavior with different environment configurations

## Usage

1. Run the application
2. A centered dialog will appear with screenshot options
3. Click on the desired screenshot type:
   - **All monitors**: Captures everything
   - **Focused monitor**: Captures the current monitor
   - **Focused window**: Captures the current window
   - **Select area**: Allows you to select a specific area
4. Screenshots are saved to `~/Pictures/Screenshots/` with timestamp filenames
5. A notification will appear allowing you to open the file or directory

## Keyboard Shortcuts

- **Escape**: Close the screenshot dialog
- Click outside the dialog to close it

## File Structure

```
screenshot-app/
├── Cargo.toml          # Rust dependencies and project config
├── src/
│   ├── main.rs         # Main application code
│   └── style.css       # GTK4 CSS styling
├── flake.nix           # Nix flake for development environment
├── .envrc              # direnv configuration for Nix
└── README.md           # This file
```

## Installation

To install the app system-wide, you can create a symlink to your desired location:

```bash
nix build .
ln -s $(pwd)/result/bin/screenshot <install_location>
```

For example, to install to `~/.local/bin`:
```bash
ln -s $(pwd)/result/bin/screenshot ~/.local/bin/screenshot
```

OR run:
```
nix profile install .
nix profile upgrade screenshot # if app already installed
```

## Notes

- The application is designed for Wayland compositors
- Hyprland-specific features (focused monitor/window) will fallback to area selection if Hyprland is not available
- The styling matches the original AGS implementation using GTK4 CSS themes
- Screenshots are saved as PNG files with timestamps in the format `YYYY.MM.DD-HH:MM:SS.png`