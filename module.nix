{withSystem, ...}: {
  flake.modules.nixos.default = {
    config,
    lib,
    pkgs,
    ...
  }: let
    cfg = config.seercat.services.low-noise-bot;
  in {
    options.seercat.services.low-noise-bot = {
      enable = lib.options.mkEnableOption "low-noise-bot";

      package = lib.options.mkPackageOption pkgs "low-noise-bot" {
        default = withSystem pkgs.stdenv.hostPlatform.system (
          {config, ...}:
            config.packages.default
        );
      };

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

        serviceConfig = {
          Restart = "on-failure";
          ExecStart = "${cfg.package}/bin/low-noise-bot";
          DynamicUser = "yes";
          # Use systemd credentials to pass the secrets to the program,
          # so the secrets can be readable by root only despite the service
          # running with DynamicUser.
          LoadCredential = ["discord_token:${cfg.discordTokenFile}" "discord_guild:${cfg.discordGuildFile}"];
        };
      };
    };
  };
}
