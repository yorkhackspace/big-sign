{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }: (let
    pkgs = nixpkgs;
  in (
    {
      nixosConfigurations."yhs-sign" = pkgs.lib.nixosSystem {
        system = "x86_64-linux";

        modules = [
          ./pi-nix-config/configuration.nix
          ({pkgs, ...}: {
            nixpkgs.overlays = [rust-overlay.overlays.default];
            environment.systemPackages = [pkgs.rust-bin.stable.latest.default];
          })
        ];
      };
    }
    // flake-utils.lib.eachDefaultSystem
    (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        yhs = pkgs.callPackage ./yhs-sign.nix {};

        nativeBuildInputs = with pkgs; [yhs.rustToolchain pkg-config libudev-zero];
        buildInputs = with pkgs; [
          yhs.rustToolchain
        ];
      in
        with pkgs; {
          formatter = pkgs.alejandra;

          devShells.default = mkShell {
            inherit buildInputs nativeBuildInputs;

            RUST_SRC_PATH = "${yhs.rustToolchain}/lib/rustlib/src/rust/library";
          };

          defaultPackage = yhs.default;
        }
    )
  ));
}
