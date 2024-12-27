
# low-noise-bot

A simple Discord bot for a private server.

For configuration, it reads the following environment variables:

- `DISCORD_TOKEN`: The bot's Discord token
- `DISCORD_GUILD`: The guild in which the bot should register commands

Alternatively, it can use the [systemd credentials](https://systemd.io/CREDENTIALS/) `discord_token` and `discord_guild`,
which will take priority over their equivalent environment variables if present.
