{ pkgs ? import <nixpkgs> { } }: let
  inherit (pkgs) rustPackages;
in pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    # Compiler and linker
    rustPackages.rustc
    clang_15
    # Native dependencies
    pkg-config
    rustPackages.cargo
    # Utilities
    cargo-deny
    cargo-watch
    cargo-outdated
    rustPackages.clippy
    rustPackages.rustfmt
  ];
  buildInputs = with pkgs; [
    openssl
    xorg.libX11
    sqlite
    libpulseaudio
  ];

  RUST_SRC_PATH = "${rustPackages.rustPlatform.rustLibSrc}";
}
