{
  pkgs,
  lib,
}: rec {
  rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  rustPlatform = pkgs.makeRustPlatform {
    rustc = rustToolchain;
    cargo = rustToolchain;
  };
  default = rustPlatform.buildRustPackage {
    inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) name;
    src = lib.cleanSource ./.;
    cargoLock.lockFile = ./Cargo.lock;
  };
}
