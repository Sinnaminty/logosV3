//! `/mimic` command suite — webhook-based persona impersonation.
//!
//! The mimic feature lets users create named personas (a display name and
//! optional avatar URL). When a mimic is active and the user sends a message,
//! the bot re-posts it via a Discord webhook so it appears to come from a
//! different identity.
//!
//! # Sub-modules
//! - [`set`] — subcommands for configuring mimic settings.
//! - [`delete`] — subcommands for removing mimics and overrides.
//!
//! # Commands in this file
//! - [`mimic`] — parent command (required by Poise).
//! - [`add`] — create a new mimic persona.
//! - [`list`] — display all of the user's mimics.
//! - [`say`] — post a one-off message as the active mimic.

use crate::commands::mimic::{delete::*, set::*};
use crate::pawthos::{
    structs::mimic::Mimic,
    types::{Context, Embed, Reply, Result},
};
use crate::utils;
use poise::serenity_prelude as serenity;
use serenity::{AutocompleteChoice, ExecuteWebhook};
mod delete;
mod set;

// ---------------------------------------------------------------------------
// Autocomplete helper
// ---------------------------------------------------------------------------

/// Provide autocomplete choices for commands that accept a mimic name.
///
/// Filters the user's mimic list by the partial string typed so far and
/// returns up to the Discord autocomplete limit of 25 entries.
async fn fetch_mimics(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    ctx.data()
        .with_mimic_user_read(ctx.author().id, |user| {
            Ok(user
                .mimics
                .iter()
                .filter_map(|m| {
                    m.name
                        .starts_with(partial)
                        .then_some(AutocompleteChoice::new(m.name.clone(), m.name.clone()))
                })
                .collect())
        })
        .await
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Mimic suite of commands — create personas and talk as them via webhook.
#[poise::command(slash_command, subcommands("add", "list", "delete", "set", "say"))]
pub async fn mimic(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Create a new mimic persona from a name and an optional avatar.
///
/// The new mimic is immediately set as your active mimic. You can provide the
/// avatar as a URL, a file attachment, or neither (the webhook uses its own
/// default avatar). Attachment takes priority over URL if both are supplied.
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Name for this mimic"] name: String,
    #[description = "Avatar URL (optional)"] avatar_url: Option<String>,
    #[description = "Attachment avatar (optional; overrides URL if given)"] attachment: Option<
        serenity::Attachment,
    >,
) -> Result {
    let user_id = ctx.author().id;

    let att_url = attachment.as_ref().map(|a| a.url.clone());
    let avatar_url = att_url.or(avatar_url);

    ctx.data()
        .with_mimic_user_write(user_id, |user| {
            let m = Mimic {
                name: name.clone(),
                avatar_url,
            };
            user.add_mimic(m.clone());
            user.active_mimic = Some(m);
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Mimic Add",
        format!("Success! Your mimic \"{}\" has been added :3c", name),
    ))
    .await?;
    Ok(())
}

/// List all of your mimics, showing each one's name and avatar.
///
/// Each mimic appears as its own embed. The first embed is a header; avatars
/// are shown as embed images where available.
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    let reply = ctx
        .data()
        .with_mimic_user_read(user_id, |user| {
            Ok(user
                .mimics
                .iter()
                .map(|m| {
                    let mut embed = Embed::new().title(m.name.clone());
                    if let Some(url) = m.avatar_url.clone() {
                        embed = embed.image(url);
                    }
                    embed
                })
                .fold(
                    utils::reply_info("Mimic List", ""),
                    |r, e| r.embed(e),
                ))
        })
        .await?;

    ctx.send(reply).await?;
    Ok(())
}

/// Post a single message in this channel as your active mimic (or channel override).
///
/// The confirmation reply ("sent~") is sent ephemerally and then immediately
/// deleted so only the webhook message is visible.
#[poise::command(slash_command)]
pub async fn say(
    ctx: Context<'_>,
    #[description = "What should your mimic say?"] text: String,
) -> Result {
    let user_id = ctx.author().id;
    let channel_id = ctx.channel_id();
    let selected_mimic = ctx
        .data()
        .with_mimic_user_read(user_id, |user| Ok(user.get_active_mimic(channel_id)))
        .await??;

    let webhook = utils::get_or_create_webhook(ctx.http(), channel_id).await?;

    let mut builder = ExecuteWebhook::new()
        .content(text)
        .username(selected_mimic.name);

    if let Some(url) = selected_mimic.avatar_url {
        builder = builder.avatar_url(url);
    }

    webhook.execute(ctx.http(), false, builder).await?;
    let reply_handle = ctx
        .send(Reply::default().ephemeral(true).content("sent~"))
        .await?;

    //delete the message :3c
    reply_handle.delete(ctx).await?;
    Ok(())
}
