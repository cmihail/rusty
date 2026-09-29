# Installation Guide

## Using the Install Script (Recommended)

This repository includes an install script that manages the screenshot application via Nix profiles.

### Quick Install
```bash
./install.sh
```

### Available Commands

#### Install or Upgrade
```bash
./install.sh install    # or just ./install.sh
```
- Installs the screenshot application if not present
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
- Removes the screenshot application from your Nix profile
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

### Manual Installation

If you prefer manual installation:

```bash
# Install
nix profile install .

# Upgrade
nix profile upgrade screenshot

# Remove
nix profile remove screenshot
```

### Troubleshooting

**"screenshot not found in PATH"**
- Restart your shell or run: `source ~/.nix-profile/etc/profile.d/nix.sh`

**"Nix profiles not supported"**
- Update to Nix 2.4+ with flakes enabled

**"Git tree is dirty" warning**
- This is just a warning and won't prevent installation
- To clean: commit your changes with `git add -A && git commit -m "message"`