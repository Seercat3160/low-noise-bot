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
      imports = [inputs.flake-parts.flakeModules.modules];
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

      flake = {
        modules.nixos.default = {
          config,
          lib,
          pkgs,
          ...
        }: let
          cfg = config.seercat.services.low-noise-bot;
        in {
          options.seercat.services.low-noise-bot = {
            enable = lib.options.mkEnableOption "low-noise-bot";

            discordTokenFile = lib.options.mkOption {
              type = lib.types.nullOr (
                lib.types.str
                // {
                  # We don't want users to be able to pass a path literal here but
                  # it should look like a path.
                  check = it: lib.isString it && lib.types.path.check it;
                }
              );
              default = null;
              example = "/run/secrets/low-noise-bot/token";
              description = ''
                Path of a file containing the bot's Discord token. The file contents are not added to the nix store.
              '';
            };

            discordGuildFile = lib.options.mkOption {
              type = lib.types.nullOr (
                lib.types.str
                // {
                  # We don't want users to be able to pass a path literal here but
                  # it should look like a path.
                  check = it: lib.isString it && lib.types.path.check it;
                }
              );
              default = null;
              example = "/run/secrets/low-noise-bot/guild_id";
              description = ''
                Path of a file containing the ID of the Discord guild the bot is in. The file contents are not added to the nix store.
              '';
            };
          };

          config = lib.mkIf cfg.enable {
            assertions = [
              {
                assertion = cfg.discordTokenFile != null;
                message = "low-noise-bot: A Discord token must be provided";
              }
              {
                assertion = cfg.discordGuildFile != null;
                message = "low-noise-bot: A Discord guild ID must be provided";
              }
            ];

            systemd.services."seercat.low-noise-bot" = {
              wantedBy = ["multi-user.target"];

              serviceConfig = let
                pkg = config.packages.default;
              in {
                Restart = "on-failure";
                ExecStart = "${pkg}/bin/low-noise-bot";
                DynamicUser = "yes";
                # Use systemd credentials to pass the secrets to the program,
                # so the secrets can be readable by root only despite the service
                # running with DynamicUser.
                LoadCredential = ["discord_token:${cfg.discordTokenFile}" "discord_guild:${cfg.discordGuildFile}"];
              };
            };
          };
        };
      };
    };
}
