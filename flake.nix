{
  description = "A basic flake for my Bevy Game";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;

        config = {
          allowUnfree = true;
        };
      };

      rustNightly = pkgs.rust-bin.nightly.latest.default.override {
        extensions = ["rust-src" "clippy" "rustfmt"];
      };
      manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
    in {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = manifest.name;
        version = manifest.version;
        cargoLock.lockFile = ./Cargo.lock;
        src = pkgs.lib.cleanSource ./.;
      };

      devShells.default = pkgs.mkShell rec {
        nativeBuildInputs = with pkgs; [
          openssl
          trunk
          wasm-pack
          rustNightly
          clippy
          rust-analyzer
          pkg-config
          llvmPackages.bintools
        ];

        buildInputs = with pkgs; [
          udev
          alsa-lib-with-plugins
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr # To use the x11 feature
          libxkbcommon
          wayland # To use the wayland feature
        ];

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        RUST_BACKTRACE = 1;

        RUST_SRC_PATH = "${rustNightly}/lib/rustlib/src/rust/library";
        #         RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };
    });
}
