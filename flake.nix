{
  description = "low-noise-bot";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    nixpkgs-mozilla = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux"];
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        pkgs = (import inputs.nixpkgs) {
          inherit system;

          overlays = [
            (import inputs.nixpkgs-mozilla)
          ];
        };

        toolchain =
          (pkgs.rustChannelOf {
            rustToolchain = ./rust-toolchain.toml;
            sha256 = "sha256-yMuSb5eQPO/bHv+Bcf/US8LVMbf/G/0MSfiPwBhiPpk=";
          })
          .rust;

        naersk' = pkgs.callPackage inputs.naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
      in rec {
        packages.default = naersk'.buildPackage {
          src = ./.;
        };

        packages.container = pkgs.dockerTools.streamLayeredImage {
          name = "low-noise-bot";
          tag = "latest-${system}";
          config.Cmd = "${packages.default}/bin/low-noise-bot";
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [toolchain];
        };

        formatter = pkgs.alejandra;
      };
    };
}
