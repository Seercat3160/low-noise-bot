use modules::{pin::PinMessageModule, BotModule};
use poise::{
    serenity_prelude::{self as serenity, GuildId},
    Framework, FrameworkError,
};
use thiserror::Error;
use tracing::{debug, error, info, warn};

type BoxedError = Box<dyn std::error::Error + Send + Sync>;
type BotContext<'a> = poise::Context<'a, BotData, BoxedError>;

pub(crate) mod modules;
pub(crate) mod util;

/// Global data accessible to all command handlers
#[derive(Debug)]
struct BotData {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing setup
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    "low_noise_bot=info"
                        .parse()
                        .expect("hard-coded default env-filter directive should be valid"),
                )
                .from_env_lossy(),
        )
        .init();

    info!("Starting low-noise-bot {}", env!("CARGO_PKG_VERSION"));

    let mut token = std::env::var("DISCORD_TOKEN").ok();
    // If set, commands will be registered in this guild rather than globally
    let debug_guild_id: Option<GuildId> = std::env::var("DISCORD_GUILD")
        .ok()
        .and_then(|x| x.parse().ok())
        .map(|x| GuildId::new(x))
        .inspect(|x| info!("Debug guild set: {x}"));

    // read values from [systemd credentials](https://systemd.io/CREDENTIALS/), overriding those set in environment variables
    if let Ok(systemd_cred_path) = std::env::var("CREDENTIALS_DIRECTORY") {
        // if file `$path/discord_token` exists, use its contents as the discord token
        if let Ok(token_string) =
            std::fs::read_to_string(format!("{systemd_cred_path}/discord_token"))
        {
            token = Some(token_string);
        } else {
            warn!("systemd credentials are in use ($CREDENTIALS_DIRECTORY = {systemd_cred_path}), but discord token was not passed that way!");
        }
    }

    let Some(token) = token else {
        error!("No Discord token provided! Set it in the DISCORD_TOKEN environment variable.");
        return Err(BotError::NoDiscordToken.into());
    };

    let modules: Vec<Box<dyn BotModule>> = vec![Box::new(PinMessageModule)];

    // set intents as the union of those required by the modules in use, and at least all the non-privileged ones
    let intents = modules
        .iter()
        .map(|m| m.gateway_intents())
        .fold(serenity::GatewayIntents::non_privileged(), |x, y| {
            x.union(y)
        });

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: modules.iter().map(|m| m.commands()).fold(vec![], |x, y| {
                let mut temp = vec![];
                temp.extend(x);
                temp.extend(y);
                temp
            }),
            on_error: |err| Box::pin(bot_error_handler(err)),
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(bot_framework_setup(ctx, framework, debug_guild_id))
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;

    tokio::spawn({
        let sm = client.shard_manager.clone();
        async move {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};

                let mut signal_terminate = signal(SignalKind::terminate()).unwrap();
                let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();

                tokio::select! {
                    _ = signal_terminate.recv() => tracing::debug!("Received SIGTERM."),
                    _ = signal_interrupt.recv() => tracing::debug!("Received SIGINT."),
                };
            }
            #[cfg(not(unix))]
            {
                tokio::signal::ctrl_c().await.unwrap();
            }

            info!("Received termination signal, shutting down!");
            sm.shutdown_all().await;
        }
    });

    client.start().await?;

    // clean up after ourselves by unregistering commands on shutdown
    // we only do this for guild-specific commands because they are used for testing,
    // while global commands should be long-lived to allow guild admins to set permissions
    // for them and have those remain functional
    if let Some(debug_guild_id) = debug_guild_id {
        info!("Unregistering debug guild commands");
        for command in client.http.get_guild_commands(debug_guild_id).await? {
            client
                .http
                .delete_guild_command(debug_guild_id, command.id)
                .await?;
        }
    }

    // also unregister commands for all guilds we're in (or at least have cached)
    // this ensures there aren't any old guild-specific commands sitting around
    for guild in client.cache.guilds() {
        debug!("unregistering commands in guild {guild}");
        for command in client.http.get_guild_commands(guild).await? {
            debug!("unregistering command {}", command.id);
            client.http.delete_guild_command(guild, command.id).await?;
        }
    }

    return Ok(());
}

async fn bot_framework_setup(
    ctx: &serenity::client::Context,
    framework: &Framework<BotData, BoxedError>,
    debug_guild_id: Option<GuildId>,
) -> Result<BotData, BoxedError> {
    // register new commands
    let new_command_names: Vec<String> = framework
        .options()
        .commands
        .iter()
        .map(|c| c.context_menu_name.clone().unwrap_or(c.name.clone()))
        .collect();

    if let Some(debug_guild_id) = debug_guild_id {
        // delete existing commands which we don't have defined anymore
        for command in ctx.http.get_guild_commands(debug_guild_id).await? {
            debug!("got existing guild command with name {:?}", command.name);

            if new_command_names.contains(&command.name) {
                debug!("not deleting");
            } else {
                debug!("deleting");
                ctx.http
                    .delete_guild_command(debug_guild_id, command.id)
                    .await?;
            }
        }

        info!("Registering commands for the debug guild");
        poise::builtins::register_in_guild(ctx, &framework.options().commands, debug_guild_id)
            .await?;
    } else {
        // delete existing commands which we don't have defined anymore
        for command in ctx.http.get_global_commands().await? {
            debug!("got existing global command with name {:?}", command.name);

            if new_command_names.contains(&command.name) {
                debug!("not deleting");
            } else {
                debug!("deleting");
                ctx.http.delete_global_command(command.id).await?;
            }
        }

        info!("Registering commands globally");
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;
    }
    info!("Registered commands");

    Ok(BotData {})
}

#[allow(clippy::unused_async)]
async fn bot_error_handler(err: FrameworkError<'_, BotData, BoxedError>) {
    match err {
        FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            warn!(
                "The bot is missing permissions in guild {:?}: {}",
                ctx.guild_id().map(poise::serenity_prelude::GuildId::get),
                missing_permissions.get_permission_names().join(",")
            );
            // Reply to the user with an error message
            let _ = ctx
                .send(
                    failure_response_embed_reply!(format!(
                        "The bot is missing permissions! Required additional permissions: `{}`",
                        missing_permissions.get_permission_names().join(" ")
                    ))
                    .ephemeral(false),
                )
                .await;
        }
        _ => {
            error!("Bot Error: {:?}", err);
        }
    }
}

#[derive(Debug, Error)]
enum BotError {
    #[error("No Discord token was provided")]
    NoDiscordToken,
}
