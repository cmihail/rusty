#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PACKAGE_NAME="rusty-hot-corner"
PROFILE_NAME="hot-corner"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_nix() {
    if ! command -v nix &> /dev/null; then
        log_error "Nix is not installed. Please install Nix first:"
        echo "  curl -L https://nixos.org/nix/install | sh"
        exit 1
    fi

    if ! nix --version &> /dev/null; then
        log_error "Nix command failed. Please check your Nix installation."
        exit 1
    fi

    log_info "Nix found: $(nix --version)"
}

check_flake_support() {
    if ! nix profile --help &> /dev/null; then
        log_error "Nix profiles not supported. Please update to Nix 2.4+ with flakes enabled."
        exit 1
    fi
}

is_installed() {
    # Check if the exact package name exists in the profile
    nix profile list 2>/dev/null | grep "^Name:" | awk '{print $2}' | sed 's/\x1b\[[0-9;]*m//g' | grep -q "^${PROFILE_NAME}$" || return 1
}

get_profile_index() {
    # In newer Nix versions, profiles use names instead of numeric indices
    # Find the entry with the exact package name and extract it
    # Format: "Name:               hot-corner" (with ANSI color codes)
    nix profile list 2>/dev/null | grep "^Name:" | awk '{print $2}' | sed 's/\x1b\[[0-9;]*m//g' | grep "^${PROFILE_NAME}$" | head -n 1
}

install_package() {
    log_info "Installing $PACKAGE_NAME..."

    if is_installed; then
        log_warning "$PACKAGE_NAME is already installed"
        read -p "Do you want to upgrade it? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            log_info "Upgrading $PACKAGE_NAME..."
            PROFILE_INDEX=$(get_profile_index)
            if [[ -z "$PROFILE_INDEX" ]]; then
                log_error "Could not find profile index for $PACKAGE_NAME"
                exit 1
            fi
            if nix profile upgrade "$PROFILE_INDEX" --impure; then
                log_success "$PACKAGE_NAME upgraded successfully!"
            else
                log_error "Failed to upgrade $PACKAGE_NAME"
                exit 1
            fi
        else
            log_info "Installation cancelled"
            exit 0
        fi
    else
        log_info "Installing $PACKAGE_NAME from $SCRIPT_DIR..."
        if nix profile install "$SCRIPT_DIR" --impure; then
            log_success "$PACKAGE_NAME installed successfully!"
        else
            log_error "Failed to install $PACKAGE_NAME"
            exit 1
        fi
    fi
}

verify_installation() {
    log_info "Verifying installation..."

    if command -v rusty-hot-corner &> /dev/null; then
        log_success "$PACKAGE_NAME is available in PATH"
        log_info "Installation location: $(which rusty-hot-corner)"
    else
        log_warning "$PACKAGE_NAME not found in PATH"
        log_info "You may need to restart your shell or run:"
        echo "  source ~/.nix-profile/etc/profile.d/nix.sh"
    fi

    # List installed packages
    log_info "Currently installed packages:"
    nix profile list | grep -A5 "^Name:.*$PROFILE_NAME" || log_warning "No $PACKAGE_NAME packages found in profile"
}

show_usage() {
    cat << EOF
Usage: $0 [COMMAND]

COMMANDS:
    install     Install or upgrade the rusty-hot-corner application (default)
    uninstall   Remove the rusty-hot-corner application from Nix profile
    status      Show installation status
    help        Show this help message

EXAMPLES:
    $0               # Install or upgrade
    $0 install       # Install or upgrade
    $0 uninstall     # Remove from profile
    $0 status        # Check if installed
EOF
}

uninstall_package() {
    log_info "Checking for installed $PACKAGE_NAME packages..."

    if ! is_installed; then
        log_warning "$PACKAGE_NAME is not installed"
        exit 0
    fi

    log_warning "This will remove $PACKAGE_NAME from your Nix profile"
    read -p "Are you sure? (y/N): " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Removing $PACKAGE_NAME..."
        PROFILE_INDEX=$(get_profile_index)
        if [[ -z "$PROFILE_INDEX" ]]; then
            log_error "Could not find profile index for $PACKAGE_NAME"
            exit 1
        fi
        if nix profile remove "$PROFILE_INDEX"; then
            log_success "$PACKAGE_NAME removed successfully!"
        else
            log_error "Failed to remove $PACKAGE_NAME"
            exit 1
        fi
    else
        log_info "Uninstallation cancelled"
    fi
}

show_status() {
    log_info "Checking $PACKAGE_NAME installation status..."

    if is_installed; then
        log_success "$PACKAGE_NAME is installed"
        nix profile list | grep -A5 "^Name:.*$PROFILE_NAME"

        if command -v rusty-hot-corner &> /dev/null; then
            log_info "Available in PATH: $(which rusty-hot-corner)"
        else
            log_warning "Not available in PATH (may need shell restart)"
        fi
    else
        log_warning "$PACKAGE_NAME is not installed"
    fi
}

main() {
    case "${1:-install}" in
        "install")
            check_nix
            check_flake_support
            install_package
            verify_installation
            ;;
        "uninstall")
            check_nix
            check_flake_support
            uninstall_package
            ;;
        "status")
            check_nix
            show_status
            ;;
        "help"|"-h"|"--help")
            show_usage
            ;;
        *)
            log_error "Unknown command: $1"
            echo
            show_usage
            exit 1
            ;;
    esac
}

main "$@"
