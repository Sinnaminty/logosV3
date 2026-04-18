//! Shared utility functions and traits.
//!
//! Contains the [`ResultExt`] helper trait, the standard embed builder, the
//! three high-level reply helpers ([`reply_ok`], [`reply_err`], [`reply_info`]),
//! and the webhook fetch-or-create helper used by the mimic feature.

use crate::pawthos::consts::{TAB_EMOJI_ID, TAB_EMOJI_NAME};
use crate::pawthos::enums::embed_type::EmbedType;
use crate::pawthos::types::{Embed, Error, Reply};
use poise::serenity_prelude as serenity;
use serenity::{EmojiId, ReactionType, Webhook};
use std::fmt::Display;

// ---------------------------------------------------------------------------
// ResultExt
// ---------------------------------------------------------------------------

/// Extension methods on [`Result`] for unrecoverable error paths.
pub trait ResultExt<T, E> {
    /// Unwrap the `Ok` value, or log the error at `ERROR` level and panic.
    ///
    /// Use this only for truly unrecoverable situations at startup (e.g. the
    /// Discord client failing to build). Prefer `?` everywhere else.
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

// ---------------------------------------------------------------------------
// Reply helpers
// ---------------------------------------------------------------------------

/// Build a "success" reply with a green embed.
///
/// Shorthand for `Reply::default().embed(create_embed_builder(title, body, EmbedType::Good))`.
pub fn reply_ok(title: impl Into<String>, body: impl Into<String>) -> Reply {
    Reply::default().embed(create_embed_builder(title, body, EmbedType::Good))
}

/// Build an "error" reply with a red embed.
///
/// Shorthand for `Reply::default().embed(create_embed_builder(title, body, EmbedType::Bad))`.
pub fn reply_err(title: impl Into<String>, body: impl Into<String>) -> Reply {
    Reply::default().embed(create_embed_builder(title, body, EmbedType::Bad))
}

/// Build an "informational" reply with a neutral (pink) embed.
///
/// Shorthand for `Reply::default().embed(create_embed_builder(title, body, EmbedType::Neutral))`.
pub fn reply_info(title: impl Into<String>, body: impl Into<String>) -> Reply {
    Reply::default().embed(create_embed_builder(title, body, EmbedType::Neutral))
}

// ---------------------------------------------------------------------------
// Embed builder
// ---------------------------------------------------------------------------

/// Build a standard Logos embed with consistent footer, author, and timestamp.
///
/// All three reply helpers delegate here; call this directly only when you
/// need to further customise the embed (e.g. add `.image()`).
///
/// # Parameters
/// - `title` — the embed title.
/// - `description` — the embed body text.
/// - `embed_type` — controls the accent colour (see [`EmbedType`]).
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

// ---------------------------------------------------------------------------
// Webhook helpers
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Reaction helpers
// ---------------------------------------------------------------------------

/// Build a [`ReactionType`] for the custom tab emoji.
///
/// Used by the faucet spawn / claim / cleanup paths to add or remove the
/// bot's tab reaction on a message.
pub fn tab_reaction() -> ReactionType {
    ReactionType::Custom {
        animated: false,
        id: EmojiId::new(TAB_EMOJI_ID),
        name: Some(TAB_EMOJI_NAME.to_string()),
    }
}

/// Return `true` if `reaction` is the tab custom emoji (matched by ID only).
///
/// Names may diverge between servers if the emoji is copied elsewhere; the
/// numeric ID is the stable identity.
pub fn is_tab_reaction(reaction: &ReactionType) -> bool {
    matches!(reaction, ReactionType::Custom { id, .. } if id.get() == TAB_EMOJI_ID)
}

// ---------------------------------------------------------------------------
// Webhook helpers
// ---------------------------------------------------------------------------

/// Return the existing `"pawthos-mimic"` webhook for `channel_id`, or create
/// one if none exists.
///
/// The mimic and auto-mode features use webhooks to post messages that appear
/// to come from a different username/avatar. A single named webhook per
/// channel is reused across all calls to avoid hitting Discord's webhook
/// creation rate limits.
pub async fn get_or_create_webhook(
    http: &serenity::Http,
    channel_id: serenity::ChannelId,
) -> Result<Webhook, Error> {
    const WEBHOOK_NAME: &str = "pawthos-mimic";
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
