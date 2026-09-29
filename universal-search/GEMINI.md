# Universal Search (`universal-search`)

## Project Overview

This is a GTK4-based universal search application written in Rust. It provides a desktop application launcher with fuzzy search capabilities, and a calculator. The project is structured as a Rust library with a binary executable.

**Technologies:**
*   **Language:** Rust
*   **UI:** GTK4
*   **Libraries:** glib, gio

**Architecture:**
*   The main application logic is in `src/main.rs`, which handles the UI and event loop.
*   `src/apps.rs` contains the logic for finding, scoring, and launching applications.
*   `src/calculator.rs` uses the `gnome-calculator` command-line tool to perform calculations.
*   `src/lib.rs` exports the `apps` and `calculator` modules, making the project a library.

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
sudo dnf install rust cargo

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

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Format and lint
cargo fmt
cargo clippy -- -D warnings
```

## Development Conventions

*   The project follows standard Rust conventions.
*   The code is well-documented with comments.
*   The `calculator` module uses a `CalculatorBackend` trait to allow for mocking in tests.
*   Use `cargo fmt` to format the code.
*   Use `cargo clippy` to lint the code.
