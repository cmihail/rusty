{
  description = "A GTK4 universal search application written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        buildInputs = with pkgs; [
          gtk4
          glib
          gdk-pixbuf
          pango
          cairo
          atk
          pkg-config
          wrapGAppsHook4
          openssl
        ];

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          wrapGAppsHook4
          openssl.dev
        ];

      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rusty-universal-search";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs;

          # Disable tests that require a display
          doCheck = false;

          # Ensure GTK4 schemas are available
          preFixup = ''
            gappsWrapperArgs+=(
              --prefix XDG_DATA_DIRS : ${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}
            )
          '';

          meta = with pkgs.lib; {
            description = "A GTK4 universal search application";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
            rust-analyzer
            rustfmt
            clippy
          ]);

          shellHook = ''
            export RUST_SRC_PATH=${rustToolchain}/lib/rustlib/src/rust/library
            export PKG_CONFIG_PATH="${pkgs.gtk4.dev}/lib/pkgconfig:${pkgs.glib.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
          '';

          # Make sure GTK can find schemas
          XDG_DATA_DIRS = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}:$XDG_DATA_DIRS";
        };
      });
}