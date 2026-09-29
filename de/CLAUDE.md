# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a GTK4-based desktop environment bar written in Rust, designed to work with Hyprland. It provides system monitoring, control center widgets, and notifications through a layer-shell bar that appears on all monitors.

## Build & Test Commands

### Basic Commands
```bash
# Build the project
cargo build

# Run the application
cargo run

# Run all tests
cargo test

# Run specific test file
cargo test --test ethernet_tests
cargo test --test wired_content_tests

# Run single test
cargo test test_ethernet_device_creation

# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Build release
cargo build --release

# Run comprehensive test script
./test.sh
```

### Nix Development
```bash
# Enter development shell
nix develop

# Build with Nix
nix build
```

## Architecture

### Service Layer (`src/service/`)
Services use the **singleton pattern** implemented via `thread_local!` with `OnceCell`. All services must be accessed through `ServiceName::instance()`. Services are GObject-based and use property notification for state changes.

**Key Services:**
- **network**: Manages both ethernet and wifi services, provides primary connection tracking
- **ethernet**: Handles wired network connections via NetworkManager D-Bus, tracks multiple devices with available connections
- **wifi**: Handles wireless connections, SSID scanning, signal strength
- **notifications**: D-Bus notification daemon implementation
- **audio**: PulseAudio/PipeWire integration
- **battery/charge_threshold**: Power management and battery charge threshold control via sysfs
- **power_profiles**: System power profile switching
- **brightness**: Screen brightness control
- **hyprland**: Window manager integration (IPC socket communication)
- **fcitx**: Input method framework integration

**D-Bus Integration:**
Services use `zbus` for D-Bus communication with async/await patterns. NetworkManager services track device state changes through property streams merged with `futures_util::stream::SelectAll`.

### Widget Layer (`src/widget/`)
GTK4 widgets that compose the UI. Widgets listen to service property changes via `connect_notify_local()`.

**Key Widgets:**
- **control_center**: Main popover containing all control widgets
- **toggles**: Row of toggle buttons for quick actions
- **switch_list**: Generic list widget for displaying multiple items with switches (used by wifi, ethernet)
- **wired_content**: Displays all ethernet devices with available connections, dynamically updates when devices are added/removed
- **slider**: Generic slider widget
- **circular_progress**: Animated progress indicator

### Window Layer (`src/window/`)
Top-level windows using gtk4-layer-shell:
- **bar**: Main status bar (one per monitor), contains toggles and system info
- **on_screen_display**: Volume/brightness overlay notifications
- **confirmation_dialog**: User confirmation prompts

### Multi-Monitor Support
The application creates one bar per monitor and automatically handles monitor hotplug events. Bars are recreated when monitors are added/removed.

## Keyboard Shortcuts

### Control Center
The control center can be opened and controlled via keyboard:

**Opening the Control Center:**
- Add the following keybind to your `~/.config/hypr/hyprland.conf` (or bindings config):
  ```
  bind = $mainMod, R, exec, rusty-de --toggle-control-center
  ```
- Press `$mainMod+R` (typically Super+R) to toggle the control center popup
- The application uses D-Bus for IPC communication between the running instance and the CLI command

**Navigating the Control Center:**
- `Escape`: Close the control center
- `Tab` / `Shift+Tab`: Cycle through toggle buttons and sliders
- `Arrow Up/Down/Left/Right`: Navigate between toggles and controls
- `Enter` / `Space`: Activate the focused toggle or expand/collapse sections

When the control center is opened via keyboard, the notifications toggle (or its expander if visible) is automatically focused for immediate keyboard navigation.

**Slider Navigation:**
Sliders (volume, brightness, microphone) support keyboard navigation:
- `Arrow Keys`: Navigate to adjacent controls (moves focus away from the slider)
- `Home`: Decrease slider value by one step
- `End`: Increase slider value by one step
- `Tab` / `Shift+Tab`: Navigate to next/previous control

## Code Patterns

### Service Implementation Pattern
1. Define service struct in `imp` module with `RefCell`/`Cell` for mutable state
2. Use `#[glib::object_subclass]` and implement `ObjectSubclass`
3. Define GObject properties in `properties()` method
4. Implement singleton access via `thread_local!` + `OnceCell`
5. Set up D-Bus listeners in async context, merge property streams for multiple devices
6. Track state changes and call `notify()` for property updates

### Multi-Device Handling Pattern (Ethernet/WiFi)
Services that manage multiple devices (like ethernet) should:
1. Maintain a `Vec<DeviceStruct>` with device metadata (path, id, connection state)
2. Implement `devices()` method to expose the list
3. Provide device-specific methods like `connect_device(&path)` and `disconnect_device(&path)`
4. Implement `connect_all()`/`disconnect_all()` that iterate over all devices
5. Filter devices by availability (e.g., only show ethernet devices with available connections)
6. Register "devices" as a GObject property and notify on changes
7. Widgets should dynamically create/destroy rows based on the devices list

### Widget Pattern
1. Store GTK widget references in struct
2. Create widgets in `new()` constructor
3. Connect to service properties for reactive updates
4. Use `updating_from_service` flag (Rc<Cell<bool>>) to prevent infinite loops when updating switch states

### Reference AGS Implementation
When implementing new features, especially for network-related functionality, refer to `/home/cmihail/configs/ags` for the correct implementation patterns. The AGS codebase serves as the reference for:
- How services should expose device lists
- How widgets should iterate over multiple devices
- How connect/disconnect operations should work per-device
- Property change notification patterns

## Testing

### Test Structure
- Tests are in `tests/` directory, one file per component
- Test both service logic and widget behavior
- Only test public methods unless explicitly requested
- Integration tests use GTK main thread for widget creation (commented out in test files with setup notes)

### Running Tests
Tests can be run individually or all together. The `test.sh` script runs comprehensive checks including formatting and linting.

## Code Style

- Empty lines must not contain any spaces or tabs
- Lines must not exceed 100 characters
- Methods should be 30-40 lines maximum; break down longer methods into smaller helper functions
- Services must use singleton pattern consistently
- Async operations use tokio runtime, D-Bus uses zbus
- GTK signals use `connect_notify_local()` for property changes
- Use `glib::clone!` macro for closure captures to avoid reference issues

## Pull Requests

- PR messages should match commit messages for single commits
- For multiple commits, PR message should summarize changes
- PR messages must not contain test plans
- Commit messages should end with:
  ```
  🤖 Generated with [Claude Code](https://claude.com/claude-code)

  Co-Authored-By: Claude <noreply@anthropic.com>
  ```
