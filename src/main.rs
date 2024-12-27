use poise::{
    serenity_prelude::{self as serenity, GuildId},
    CreateReply,
};

struct Data {} // stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    println!("Discord bot is starting...");

    // read config from environment variables
    let mut token = std::env::var("DISCORD_TOKEN").ok();
    let mut guild_id = std::env::var("DISCORD_GUILD").ok();

    // read config from [systemd credentials](https://systemd.io/CREDENTIALS/), overriding environment variables if both are present
    if let Ok(systemd_cred_path) = std::env::var("CREDENTIALS_DIRECTORY") {
        // if file `$path/discord_token` exists, use it's contents as the discord token
        if let Ok(token_string) =
            std::fs::read_to_string(format!("{systemd_cred_path}/discord_token"))
        {
            token = Some(token_string);
        }
        // if file `$path/discord_guild` exists, use it's contents as the guild ID
        if let Ok(guild_id_string) =
            std::fs::read_to_string(format!("{systemd_cred_path}/discord_guild"))
        {
            guild_id = Some(guild_id_string);
        }
    }

    let token: String = token.expect("DISCORD_TOKEN should be set");
    let guild_id: u64 = guild_id
        .clone()
        .expect("DISCORD_GUILD should be set")
        .parse()
        .unwrap_or_else(|_| {
            panic!(
                "Discord guild ID provided should be valid: got '{}'",
                guild_id.unwrap()
            )
        });

    let intents = serenity::GatewayIntents::non_privileged();

    println!("Defined intents");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![pin_message(), unpin_message()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("About to register commands");
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(guild_id),
                )
                .await?;
                println!("Registered commands");
                Ok(Data {})
            })
        })
        .build();

    println!("Creating client");
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    println!("Created client");
    client.unwrap().start().await.unwrap();
}

/// Pin a message
#[poise::command(context_menu_command = "Pin Message")]
async fn pin_message(
    ctx: Context<'_>,
    #[description = "Message to pin"] message: serenity::Message,
) -> Result<(), Error> {
    println!(
        "Asked to pin message {} by user {}",
        message.id,
        ctx.author().id
    );

    if message.pinned {
        println!("Message {} is already pinned", message.id);
        ctx.send(CreateReply::default().ephemeral(true).content(format!(
            "📌 [That message]({}) is already pinned!",
            message.link()
        )))
        .await?;
    } else {
        match message.pin(ctx).await {
            Ok(()) => {
                println!(
                    "Pinned message {} at request of user {}",
                    message.id,
                    ctx.author().id
                );
                ctx.send(
                    CreateReply::default()
                        .content(format!("📌 Pinned [message]({})", message.link())),
                )
                .await?;
            }
            Err(e) => {
                println!("Failed to pin message {}: {}", message.id, e);
                ctx.send(
                    CreateReply::default()
                        .ephemeral(true)
                        .content(format!("📌 Failed to pin [message]({})!", message.link())),
                )
                .await?;
            }
        };
    }

    Ok(())
}

/// Unpin a message
#[poise::command(context_menu_command = "Unpin Message")]
async fn unpin_message(
    ctx: Context<'_>,
    #[description = "Message to unpin"] message: serenity::Message,
) -> Result<(), Error> {
    println!(
        "Asked to unpin message {} by user {}",
        message.id,
        ctx.author().id
    );

    if message.pinned {
        match message.unpin(ctx).await {
            Ok(()) => {
                println!(
                    "Unpinned message {} at request of user {}",
                    message.id,
                    ctx.author().id
                );
                ctx.send(
                    CreateReply::default()
                        .content(format!("📌 Unpinned [message]({})", message.link())),
                )
                .await?;
            }
            Err(e) => {
                println!("Failed to unpin message {}: {}", message.id, e);
                ctx.send(
                    CreateReply::default()
                        .ephemeral(true)
                        .content(format!("📌 Failed to unpin [message]({})!", message.link())),
                )
                .await?;
            }
        };
    } else {
        println!("Message {} is already not pinned", message.id);
        ctx.send(CreateReply::default().ephemeral(true).content(format!(
            "📌 [That message]({}) is not pinned!",
            message.link()
        )))
        .await?;
    }
    Ok(())
}
