{
  description = "low-noise-bot";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [inputs.flake-parts.flakeModules.modules ./module.nix];
      systems = ["x86_64-linux" "aarch64-linux"];
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        toolchain = inputs.fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-vra6TkHITpwRyA5oBKAHSX0Mi6CBDNQD+ryPSpxFsfg=";
        };

        naersk' = pkgs.callPackage inputs.naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
      in rec {
        packages.default = naersk'.buildPackage {
          src = ./.;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [toolchain];
        };

        formatter = pkgs.alejandra;
      };
    };
}
