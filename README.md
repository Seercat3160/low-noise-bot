
# low-noise-bot

A simple Discord bot for a private server.

For configuration, it reads the following environment variable:

- `DISCORD_TOKEN`: The bot's Discord token

Alternatively, it can use the [systemd credential](https://systemd.io/CREDENTIALS/) `discord_token`,
which will take priority over the equivalent environment variable if present.
