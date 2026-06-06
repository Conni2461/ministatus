{ pkgs ? import <nixpkgs> { } }: let
  inherit (pkgs) rustPackages;
in pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    # Compiler and linker
    rustPackages.rustc
    clang
    # Native dependencies
    pkg-config
    rustPackages.cargo
    # Utilities
    cargo-deny
    cargo-watch
    rustPackages.clippy
    rustPackages.rustfmt
  ];
  buildInputs = with pkgs; [
    xorg.libX11
    sqlite
    libpulseaudio
  ];

  RUST_SRC_PATH = "${rustPackages.rustPlatform.rustLibSrc}";
}
