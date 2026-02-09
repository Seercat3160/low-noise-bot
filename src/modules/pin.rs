use tracing::info;
use tracing::warn;

use crate::failure_response_embed_reply;
use crate::success_response_embed_reply;
use crate::BotContext;
use crate::BoxedError;

use poise::serenity_prelude as serenity;

use super::BotModule;

// See https://discord.com/developers/docs/resources/channel#pin-message
// As of 2025-02-16, it says "The max pinned messages is 50."
const MAX_PINS_PER_CHANNEL: usize = 50;

#[derive(Debug)]
pub(crate) struct PinMessageModule;

impl BotModule for PinMessageModule {
    fn commands(&self) -> Vec<poise::Command<crate::BotData, BoxedError>> {
        vec![pin_message(), unpin_message()]
    }
}

/// Pin a message
#[poise::command(
    context_menu_command = "Pin Message",
    guild_only = true,
    required_bot_permissions = "MANAGE_MESSAGES",
    default_member_permissions = "ADMINISTRATOR"
)]
async fn pin_message(
    ctx: BotContext<'_>,
    #[description = "Message to pin"] message: serenity::Message,
) -> Result<(), BoxedError> {
    info!(
        "Asked to pin message {} ({}) by user {}",
        message.id,
        message.link(),
        ctx.author().id
    );

    if message.pinned {
        info!(
            "Message {} ({}) is already pinned",
            message.id,
            message.link()
        );
        ctx.send(
            success_response_embed_reply!(format!(
                "📌 [That message]({}) is already pinned!",
                message.link()
            ))
            .ephemeral(true),
        )
        .await?;
    } else if ctx
        .http()
        .get_pins(ctx.channel_id())
        .await
        .map(|p| p.len())
        .map(|l| l >= MAX_PINS_PER_CHANNEL)
        .unwrap_or(false)
    {
        // Attempting to pin will fail as the channel is at (or above) the limit
        warn!(
            "Failed to pin message {} ({}) for user {}, the channel has too many pinned messages already (limit is {MAX_PINS_PER_CHANNEL})",
            message.id,
            message.link(),
            ctx.author().id,
        );
        ctx.send(failure_response_embed_reply!(format!(
            "📌 Can't pin [that message]({}) because this channel has too many pinned messages: the limit is {MAX_PINS_PER_CHANNEL} per channel",
            message.link()
        )))
        .await?;
    } else {
        match ctx
            .http()
            .pin_message(
                ctx.channel_id(),
                message.id,
                Some(&format!(
                    "low-noise-bot: pinning message for user {}",
                    ctx.author().id
                )),
            )
            .await
        {
            Ok(()) => {
                info!(
                    "Pinned message {} ({}) at request of user {}",
                    message.id,
                    message.link(),
                    ctx.author().id
                );
                ctx.send(success_response_embed_reply!(format!(
                    "📌 Pinned [message]({})",
                    message.link()
                )))
                .await?;
            }
            Err(e) => {
                warn!(
                    "Failed to pin message {} ({}) for user {}: {}",
                    message.id,
                    message.link(),
                    ctx.author().id,
                    e
                );
                ctx.send(failure_response_embed_reply!(format!(
                    "📌 Failed to pin [message]({})!",
                    message.link()
                )))
                .await?;
            }
        }
    }

    Ok(())
}

/// Unpin a message
#[poise::command(
    context_menu_command = "Unpin Message",
    guild_only = true,
    required_bot_permissions = "MANAGE_MESSAGES",
    default_member_permissions = "ADMINISTRATOR"
)]
async fn unpin_message(
    ctx: BotContext<'_>,
    #[description = "Message to unpin"] message: serenity::Message,
) -> Result<(), BoxedError> {
    info!(
        "Asked to unpin message {} ({}) by user {}",
        message.id,
        message.link(),
        ctx.author().id
    );

    if message.pinned {
        match ctx
            .http()
            .unpin_message(
                ctx.channel_id(),
                message.id,
                Some(&format!(
                    "low-noise-bot: unpinning message for user {}",
                    ctx.author().id
                )),
            )
            .await
        {
            Ok(()) => {
                info!(
                    "Unpinned message {} ({}) at request of user {}",
                    message.id,
                    message.link(),
                    ctx.author().id
                );
                ctx.send(success_response_embed_reply!(format!(
                    "📌 Unpinned [message]({})",
                    message.link()
                )))
                .await?;
            }
            Err(e) => {
                warn!(
                    "Failed to unpin message {} ({}) for user {}: {}",
                    message.id,
                    message.link(),
                    ctx.author().id,
                    e
                );
                ctx.send(failure_response_embed_reply!(format!(
                    "📌 Failed to unpin [message]({})!",
                    message.link()
                )))
                .await?;
            }
        }
    } else {
        info!(
            "Message {} ({}) is already not pinned",
            message.id,
            message.link()
        );
        ctx.send(
            success_response_embed_reply!(format!(
                "📌 [That message]({}) is not pinned!",
                message.link()
            ))
            .ephemeral(true),
        )
        .await?;
    }
    Ok(())
}
