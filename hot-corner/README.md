# rusty-hot-corner

A hot corner application written in Rust using GTK4 and gtk4-layer-shell, designed to work with Hyprland.

## Features

- Hot corner activation at all four corners (top-left, top-right, bottom-left, bottom-right)
- Independent command configuration for each corner
- Configurable trigger delay (default 1000ms)
- Per-corner multi-monitor support (all monitors or edge monitors only)
- Only triggers on focused monitor to prevent multi-monitor issues
- Automatic monitor hotplug support
- Live configuration reload without restart
- GTK4-based layer shell integration
- Lightweight and efficient

## Installation

### Using Nix (Recommended)

```bash
# Install or upgrade
./install.sh

# Check status
./install.sh status

# Uninstall
./install.sh uninstall
```

### Manual Build

```bash
# Build the project
cargo build --release

# Run the application
cargo run
```

## Configuration

The application reads its configuration from `~/.config/rusty/hot-corner.toml` and automatically reloads when the file changes.

### Configuration File

Create the configuration file at `~/.config/rusty/hot-corner.toml`:

```toml
# Top-left corner
top_left_corner_command = "hyprctl dispatch hyprexpo:expo"
top_left_corner_on_all_monitors = false

# Top-right corner
top_right_corner_command = "notify-send 'Top right corner!'"
top_right_corner_on_all_monitors = false

# Bottom-left corner
bottom_left_corner_command = "notify-send 'Bottom left corner!'"
bottom_left_corner_on_all_monitors = false

# Bottom-right corner
bottom_right_corner_command = "rofi -show drun"
bottom_right_corner_on_all_monitors = true

# Global trigger delay (optional, default: 1000ms)
trigger_delay_ms = 1000
```

### Configuration Options

#### Corner Commands

Each corner can be configured independently:

- **top_left_corner_command**: Command to execute when the top-left corner is triggered
- **top_right_corner_command**: Command to execute when the top-right corner is triggered
- **bottom_left_corner_command**: Command to execute when the bottom-left corner is triggered
- **bottom_right_corner_command**: Command to execute when the bottom-right corner is triggered

#### Multi-Monitor Behavior

Control whether hot corners appear on all monitors or only on edge monitors:

- **top_left_corner_on_all_monitors**: `true` = all monitors, `false` = leftmost monitor only (default: `false`)
- **top_right_corner_on_all_monitors**: `true` = all monitors, `false` = rightmost monitor only (default: `false`)
- **bottom_left_corner_on_all_monitors**: `true` = all monitors, `false` = leftmost monitor only (default: `false`)
- **bottom_right_corner_on_all_monitors**: `true` = all monitors, `false` = rightmost monitor only (default: `false`)

#### Global Settings

- **trigger_delay_ms**: Delay in milliseconds before triggering the command after hovering (default: `1000`)

### Example Configurations

#### Single Monitor Setup

```toml
top_left_corner_command = "hyprctl dispatch hyprexpo:expo"
top_right_corner_command = "notify-send 'Top right!'"
bottom_left_corner_command = "rofi -show drun"
bottom_right_corner_command = "notify-send 'Bottom right!'"
```

#### Multi-Monitor Setup (Edge Corners Only)

```toml
# Top-left on leftmost monitor only
top_left_corner_command = "hyprctl dispatch hyprexpo:expo"
top_left_corner_on_all_monitors = false

# Top-right on rightmost monitor only
top_right_corner_command = "notify-send 'Right edge!'"
top_right_corner_on_all_monitors = false
```

#### Multi-Monitor Setup (All Monitors)

```toml
# Top-left on all monitors
top_left_corner_command = "hyprctl dispatch hyprexpo:expo"
top_left_corner_on_all_monitors = true

# Bottom-right on all monitors
bottom_right_corner_command = "rofi -show drun"
bottom_right_corner_on_all_monitors = true
```

#### Custom Trigger Delay

```toml
# Faster trigger (500ms)
top_left_corner_command = "hyprctl dispatch hyprexpo:expo"
trigger_delay_ms = 500
```

#### Minimal Configuration

```toml
# Only configure the corners you want to use
top_left_corner_command = "hyprctl dispatch hyprexpo:expo"
```

**Note**: Corners without a command configured will not be created.

## Development

### Build Commands

```bash
# Build
cargo build

# Run
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

### Nix Development Shell

```bash
nix develop
```

## License

MIT
