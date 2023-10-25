{ pkgs ? import <nixpkgs> {} }: let
  unstable = pkgs.unstable or (import <nixpkgs-unstable> { });
  rustPackages = pkgs.rustPackages_1_66 or pkgs.rustPackages;
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
    xorg.libX11
    sqlite
  ];

  RUST_SRC_PATH = "${rustPackages.rustPlatform.rustLibSrc}";
}
