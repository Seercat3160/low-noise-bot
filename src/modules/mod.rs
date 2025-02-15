use poise::serenity_prelude::{self as serenity, GatewayIntents};

use crate::{BotData, BoxedError};

pub(crate) mod pin;

pub(crate) trait BotModule {
    /// The Application Commands that are provided by this module to be registered to the bot framework
    fn commands(&self) -> Vec<poise::Command<BotData, BoxedError>>;

    /// The Gateway Intents required by this module
    fn gateway_intents(&self) -> serenity::GatewayIntents {
        GatewayIntents::non_privileged()
    }
}
