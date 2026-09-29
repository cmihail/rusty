# rusty

A collection of GTK4 desktop tools written in Rust for the Hyprland window manager.

## Projects

### 1. Desktop Environment (`de`)
**Description**: Status bar and control center with a D-Bus notification daemon, WiFi and Ethernet management, audio, power profiles, battery charge thresholds, brightness, multi-monitor hotplug, Hyprland workspace integration and Fcitx input method support

[Full Documentation](de/README.md)

### 2. Hot Corner (`hot-corner`)
**Description**: Runs a configurable command per screen corner after a trigger delay, with per-corner monitor selection and live config reload

[Full Documentation](hot-corner/README.md)

### 3. Screenshot (`screenshot`)
**Description**: Captures all monitors, the focused monitor, the focused window or a selected area using `grim` and `slurp`, and records screen video with `wf-recorder`

[Full Documentation](screenshot/README.md)

### 4. Universal Search (`universal-search`)
**Description**: Application launcher with fuzzy search, usage-based ranking, a calculator, web search (Brave Search) and AI answers (Gemini)

[Full Documentation](universal-search/README.md)

## Requirements

- Hyprland (Wayland)
- GTK4 and gtk4-layer-shell
- Nix with flakes enabled (recommended), or a Rust toolchain for manual builds

Each project lists its extra runtime dependencies in its own README.

## Building

### Using Nix (Recommended)

Every project has an `install.sh` that builds its flake and installs it into a dedicated Nix profile:

```bash
cd <project>
./install.sh            # install or upgrade
./install.sh status     # show installation status
./install.sh uninstall
```

Install all projects:

```bash
for project in de hot-corner screenshot universal-search; do
    (cd "$project" && ./install.sh)
done
```

### Manual Build

```bash
cd <project>
cargo build --release
```

## Configuration

Configuration files live in `~/.config/rusty/`:

| Project | File |
|---------|------|
| de | `~/.config/rusty/de.toml` |
| hot-corner | `~/.config/rusty/hot-corner.toml` |

Universal Search reads its API keys from the `BRAVE_SEARCH_API_KEY` and `PERSONAL_GEMINI_API_KEY` environment variables. See [its README](universal-search/README.md) for storing them in the keyring with `secret-tool`.

## Testing

```bash
cd <project>
./test.sh
```

## License

GPL-3.0. See [LICENSE](LICENSE).
