use std::sync::LazyLock;

use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};

static EMBED_VERSION_FOOTER: LazyLock<CreateEmbedFooter> = std::sync::LazyLock::new(|| {
    CreateEmbedFooter::new(format!(
        "low-noise-bot, version {}",
        env!("CARGO_PKG_VERSION")
    ))
});

/// Get a [`CreateEmbed`] with a standard footer
pub(crate) fn generic_embed() -> ::poise::serenity_prelude::builder::CreateEmbed {
    CreateEmbed::new().footer(EMBED_VERSION_FOOTER.clone())
}

/// Get a [`CreateEmbed`] with a standard footer and success colours
pub(crate) fn success_embed() -> ::poise::serenity_prelude::builder::CreateEmbed {
    generic_embed().colour(::poise::serenity_prelude::colours::branding::GREEN)
}

/// Get a [`CreateEmbed`] with a standard footer and failure colours
pub(crate) fn failure_embed() -> ::poise::serenity_prelude::builder::CreateEmbed {
    generic_embed().colour(::poise::serenity_prelude::colours::branding::RED)
}

#[macro_export]
macro_rules! failure_response_embed_reply {
    ($body:expr) => {
        ::poise::CreateReply::default()
            .reply(true)
            .ephemeral(true)
            .embed($crate::util::failure_embed().description($body))
    };
}

#[macro_export]
macro_rules! success_response_embed_reply {
    ($body:expr) => {
        ::poise::CreateReply::default()
            .reply(true)
            .embed($crate::util::success_embed().description($body))
    };
}
