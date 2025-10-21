use std::fmt::Display;

use poise::serenity_prelude::Webhook;

use crate::types::{Embed, EmbedType, Error};
use poise::serenity_prelude::{self as serenity};

/// this is a trait!!
pub trait ResultExt<T, E> {
    /// Unwraps the result, logging the error and panicking if it's an Err.
    fn unwrap_or_log(self, from: impl Display) -> T;
}

impl<T, E: std::fmt::Display> ResultExt<T, E> for Result<T, E> {
    fn unwrap_or_log(self, from: impl Display) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                log::error!("Unrecoverable error from {from}: {e}");
                panic!();
            }
        }
    }
}

pub fn create_embed_builder(
    title: impl Into<String>,
    description: impl Into<String>,
    embed_type: EmbedType,
) -> Embed {
    Embed::new()
        .title(title)
        .description(description)
        .timestamp(serenity::Timestamp::now())
        .footer(serenity::builder::CreateEmbedFooter::new(
            "Powered by caffeine and lambda functions.",
        ))
        .author(serenity::builder::CreateEmbedAuthor::new("Logos"))
        .color(embed_type.into_color())
}

pub async fn get_or_create_webhook(
    http: &serenity::Http,
    channel_id: serenity::ChannelId,
) -> Result<Webhook, Error> {
    const WEBHOOK_NAME: &str = "logosV3-mimic";
    if let Ok(existing) = channel_id.webhooks(http).await
        && let Some(w) = existing
            .into_iter()
            .find(|w| w.name.as_deref() == Some(WEBHOOK_NAME))
    {
        return Ok(w);
    }

    //the webby don't exist :c
    let hook = channel_id
        .create_webhook(http, serenity::CreateWebhook::new(WEBHOOK_NAME))
        .await?;
    Ok(hook)
}
