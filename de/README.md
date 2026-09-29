# rusty-de

A GTK4-based desktop environment bar written in Rust, designed to work with Hyprland.

## Features

- System monitoring and control center widgets
- D-Bus notification daemon implementation
- Network management (WiFi and Ethernet)
- Audio control (PulseAudio/PipeWire)
- Power management (battery charge thresholds, power profiles)
- Screen brightness control
- Multi-monitor support with automatic hotplug detection
- Hyprland workspace integration
- Input method (Fcitx) integration

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

## Running the Application

The application is managed by systemd as a user service with automatic restart capability. This ensures the application automatically restarts if it crashes, including after system suspend/resume.

### Systemd Service Commands

```bash
# Check service status
systemctl --user status rusty-de.service

# Start the service
systemctl --user start rusty-de.service

# Stop the service
systemctl --user stop rusty-de.service

# Restart the service
systemctl --user restart rusty-de.service

# Enable automatic startup (already enabled by default)
systemctl --user enable rusty-de.service

# Disable automatic startup
systemctl --user disable rusty-de.service

# View logs in real-time
journalctl --user -u rusty-de.service -f

# View recent logs
journalctl --user -u rusty-de.service -n 50
```

### Hyprland Integration

The service is automatically started by Hyprland on login via the configuration in `~/.config/hypr/config/autostart.conf`.

You can also restart the application using the keyboard shortcut:
- **Super+R**: Restart rusty-de service

### Suspend/Resume Handling

The application automatically handles system suspend/resume:
1. When the system prepares to suspend, the app exits cleanly
2. After resume, systemd automatically restarts the service
3. This prevents "Lost connection to Wayland compositor" errors

## Configuration

### Workspace Manager

The workspace manager automatically manages workspace placement based on connected monitors. Create a configuration file at `~/.config/rusty/de.toml`:

```toml
# Workspace layout configuration
# Monitors are sorted by X position (left to right), then Y position (top to bottom)
# m1 = leftmost monitor
# m2 = second monitor from the left
# m3 = third monitor from the left

# Single monitor: all workspaces on one monitor
[layout.1]
m1 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

# Dual monitor layout
[layout.2]
m1 = [2, 5, 6]
m2 = [1, 3, 4]

# Triple monitor layout
[layout.3]
m1 = [7]
m2 = [2, 5, 6]
m3 = [1, 3, 4]
```

**Features:**
- Automatically applies layouts when monitors are connected or disconnected
- Monitors are identified by physical position (left to right, top to bottom)
- Workspaces are created and moved to their assigned monitors
- Each monitor automatically switches to its first configured workspace

### Battery Charge Threshold Service

The service writes the charge thresholds directly to sysfs, which is root-owned by default. Grant
your user write access with a systemd oneshot in
`/etc/systemd/system/battery-charge-threshold-perms.service` (replace `users` with any group you are
a member of):

```ini
[Unit]
Description=Grant the users group write access to the battery charge thresholds
After=multi-user.target
ConditionPathExists=/sys/class/power_supply/BAT0/charge_control_end_threshold

[Service]
Type=oneshot
ExecStart=/bin/chgrp users /sys/class/power_supply/BAT0/charge_control_start_threshold /sys/class/power_supply/BAT0/charge_control_end_threshold
ExecStart=/bin/chmod 0664 /sys/class/power_supply/BAT0/charge_control_start_threshold /sys/class/power_supply/BAT0/charge_control_end_threshold

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now battery-charge-threshold-perms.service
```

A udev rule matching `SUBSYSTEM=="power_supply", KERNEL=="BAT[0-9]"` is the more usual way to grant
this, but it does not work on a ThinkPad T14: udevd never processes any event for `BAT0`, so the
rule's `RUN` commands never execute. There is no `/run/udev/data/+power_supply:BAT0` entry even
though `AC` has one, `udevadm trigger --action=change` and a direct `echo change > uevent` both
return success while producing no udevd activity at debug log level, and the permissions stay
root-only. `udevadm test` is not a way to check this — it matches the rule and prints the queued
`RUN` commands, but its first line notes that it never runs them.

The unit runs after `multi-user.target` because `thinkpad_acpi` creates the `charge_control_*`
attributes late, well after udev's coldplug pass; `ConditionPathExists` skips the unit rather than
failing it if the attributes are somehow still missing. The thresholds themselves are held in the
EC and survive a reboot, but the permissions do not, which is why the unit has to run every boot.

Power profile switching requires `power-profiles-daemon`. Note that Debian and Ubuntu package it
as conflicting with `tlp`, so installing it removes TLP.

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

# Comprehensive test script
./test.sh
```

### Nix Development Shell

```bash
nix develop
```

## Architecture

- **Service Layer** (`src/service/`) - Singleton services for system integration
- **Widget Layer** (`src/widget/`) - GTK4 UI components
- **Window Layer** (`src/window/`) - Top-level windows using gtk4-layer-shell

## License

MIT
