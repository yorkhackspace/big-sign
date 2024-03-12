{
  pkgs,
  lib,
}: rec {
  rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  rustPlatform = pkgs.makeRustPlatform {
    rustc = rustToolchain;
    cargo = rustToolchain;
  };
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  buildInputs = with pkgs; [
    libudev-zero
  ];
  default = rustPlatform.buildRustPackage {
    inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) name;
    src = lib.cleanSource ./.;
    cargoLock.lockFile = ./Cargo.lock;
    inherit nativeBuildInputs buildInputs;
  };
}
