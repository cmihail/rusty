{
  description = "A GTK4 screenshot application written in Rust";

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
          gtk4-layer-shell
          glib
          gdk-pixbuf
          pango
          cairo
          atk
          pkg-config
          wrapGAppsHook4
          grim
          slurp
          hyprland
        ];

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          wrapGAppsHook4
        ];

      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rusty-screenshot";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs;

          # Ensure GTK4 schemas are available
          preFixup = ''
            gappsWrapperArgs+=(
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.grim pkgs.slurp pkgs.wf-recorder ]}
            )
          '';

          meta = with pkgs.lib; {
            description = "A GTK4 screenshot application";
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
            grim
            slurp
            libnotify
          ]);

          shellHook = ''
            export RUST_SRC_PATH=${rustToolchain}/lib/rustlib/src/rust/library
            export PKG_CONFIG_PATH="${pkgs.gtk4.dev}/lib/pkgconfig:${pkgs.glib.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
            export PKG_CONFIG_PATH="${pkgs.gtk4-layer-shell.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
          '';

          # Make sure GTK can find schemas
          XDG_DATA_DIRS = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}:$XDG_DATA_DIRS";
        };
      });
}