# Installation Guide

## Using the Install Script (Recommended)

This repository includes an install script that manages the universal search application via Nix profiles.

### Quick Install
```bash
./install.sh
```

### Available Commands

#### Install or Upgrade
```bash
./install.sh install    # or just ./install.sh
```
- Installs the universal search application if not present
- Offers to upgrade if already installed
- Verifies installation after completion

#### Check Installation Status
```bash
./install.sh status
```
- Shows whether the application is installed
- Displays installation details and PATH availability

#### Uninstall
```bash
./install.sh uninstall
```
- Removes the universal search application from your Nix profile
- Asks for confirmation before removal

#### Help
```bash
./install.sh help
```
- Shows usage information and available commands

### Features

- **Automatic Detection**: Detects if the package is already installed
- **Safe Upgrades**: Prompts before upgrading existing installations
- **Path Verification**: Checks if the binary is available in your PATH
- **Error Handling**: Provides clear error messages and troubleshooting hints
- **Colored Output**: Uses colors to distinguish info, warnings, and errors

### Requirements

- Nix package manager (2.4+ with flakes support)
- Git (for accessing the repository)
- Desktop environment (X11 or Wayland)

### Manual Installation

If you prefer manual installation:

```bash
# Install
nix profile install .

# Upgrade
nix profile upgrade universal-search

# Remove
nix profile remove universal-search
```

### Desktop Integration

To add a desktop shortcut or integrate with your desktop environment:

```bash
# Create a desktop entry
cat > ~/.local/share/applications/universal-search.desktop << EOF
[Desktop Entry]
Name=Universal Search
Comment=Quick application launcher with fuzzy search
Exec=universal-search
Icon=system-search
Terminal=false
Type=Application
Categories=Utility;
Keywords=search;launcher;apps;
EOF

# Update desktop database
update-desktop-database ~/.local/share/applications
```

### Window Manager Integration

For integration with window managers or desktop environments, you can bind the application to a keyboard shortcut:

**Hyprland example:**
```
bind = SUPER, SPACE, exec, universal-search
```

**Sway example:**
```
bindsym $mod+space exec universal-search
```

**i3 example:**
```
bindsym $mod+space exec universal-search
```

### Troubleshooting

**"universal-search not found in PATH"**
- Restart your shell or run: `source ~/.nix-profile/etc/profile.d/nix.sh`

**"Nix profiles not supported"**
- Update to Nix 2.4+ with flakes enabled

**"Git tree is dirty" warning**
- This is just a warning and won't prevent installation
- To clean: commit your changes with `git add -A && git commit -m "message"`


**Application window doesn't appear**
- Try running from a terminal to see error messages: `universal-search`

**No applications found in search**
- Verify that applications are installed and have .desktop files
- Check that `~/.local/share/applications` and `/usr/share/applications` contain .desktop files
- Run `gio list applications` to see what GIO can discover

**Styling issues**
- The app uses GTK4 CSS with system theme colors
- Ensure your desktop environment provides proper GTK4 themes
- Custom themes can be applied by modifying `src/style.css`

### Performance Tips

- The application caches application discovery for better performance
- Search results are limited to 12 items for optimal UI responsiveness
- Frequency tracking improves result relevance over time

### Uninstalling

To completely remove the application:

```bash
# Remove from Nix profile
./install.sh uninstall

# Remove desktop entry (if created)
rm ~/.local/share/applications/universal-search.desktop
update-desktop-database ~/.local/share/applications

# Remove any cached data (if any)
rm -rf ~/.cache/universal-search
```