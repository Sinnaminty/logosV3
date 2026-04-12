//! `/mimic set` subcommands — configure mimic settings.
//!
//! All three subcommands modify the calling user's [`MimicUser`] record:
//!
//! - [`active_mimic`] — choose which mimic is active by default.
//! - [`channel_override`] — pin a specific mimic to a particular channel.
//! - [`auto`] — toggle auto-mode (intercept all messages as the active mimic).
//!
//! [`MimicUser`]: crate::pawthos::structs::mimic_user::MimicUser

use crate::pawthos::{
    enums::mimic_errors::MimicError,
    types::{Context, Result},
};
use crate::{commands::mimic::fetch_mimics, utils};
use poise::serenity_prelude::Channel;

/// Mimic settings subcommands.
#[poise::command(slash_command, subcommands("active_mimic", "channel_override", "auto"))]
pub async fn set(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Set which of your mimics is the active (default) one.
///
/// The active mimic is used by `/mimic say` and by auto-mode in channels that
/// have no override configured. Autocomplete lists your existing mimics.
#[poise::command(slash_command)]
pub async fn active_mimic(
    ctx: Context<'_>,
    #[autocomplete = "fetch_mimics"] name: String,
) -> Result {
    let user_id = ctx.author().id;
    let target = name.trim();

    let mimic_name = ctx
        .data()
        .with_mimic_user_write(user_id, |user| {
            let m = user
                .mimics
                .iter()
                .find(|m| m.name == target)
                .ok_or(MimicError::MimicNotFound)?;
            user.active_mimic = Some(m.clone());

            Ok(m.name.clone())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Mimic Set active_mimic",
        format!("Your active mimic is set to \"{}\"", mimic_name),
    ))
    .await?;
    Ok(())
}

/// Pin a specific mimic to a channel, overriding the active mimic there.
///
/// When auto-mode fires (or you use `/mimic say`) in `channel`, the override
/// mimic is used instead of your active mimic. Autocomplete lists your mimics.
#[poise::command(slash_command)]
pub async fn channel_override(
    ctx: Context<'_>,
    #[description = "What channel do you want to override?"] channel: Channel,
    #[description = "What Mimic do you want to set to this channel?"]
    #[autocomplete = "fetch_mimics"]
    name: String,
) -> Result {
    let user_id = ctx.author().id;
    let channel_id = channel.id();
    let target = name.trim();

    let mimic_name = ctx
        .data()
        .with_mimic_user_write(user_id, |user| {
            let m = user
                .mimics
                .iter()
                .find(|m| m.name == target)
                .ok_or(MimicError::MimicNotFound)?;

            user.channel_override.insert(channel_id, m.clone());
            Ok(m.name.clone())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Mimic Set channel_override",
        format!("\"{}\" is set to channel \"{}\"", mimic_name, channel),
    ))
    .await?;
    Ok(())
}

/// Enable/disable choice parameter for the `auto` subcommand.
#[derive(poise::ChoiceParameter, PartialEq)]
pub enum AutoChoice {
    #[name = "Enable"]
    Enable,
    #[name = "Disable"]
    Disable,
}

/// Enable or disable auto-mode for your active mimic.
///
/// When auto-mode is **enabled**, every message you send in a guild channel is
/// intercepted: the bot re-posts it as your active mimic (via webhook) and
/// deletes the original, making it appear as though the mimic is speaking.
///
/// Auto-mode requires an active mimic to be set — the command returns an error
/// if none is configured.
#[poise::command(slash_command)]
pub async fn auto(
    ctx: Context<'_>,
    #[description = "Enable/Disable Auto mode."] choice: AutoChoice,
) -> Result {
    let user_id = ctx.author().id;
    let enable = matches!(choice, AutoChoice::Enable);

    let outcome = ctx
        .data()
        .with_mimic_user_write(user_id, |user| {
            if user.active_mimic.is_none() {
                return Err(MimicError::NoActiveMimic);
            }
            user.auto_mode = enable;
            Ok(enable)
        })
        .await?;

    ctx.send(utils::reply_ok("Mimic Auto", format!("Auto Mode: {}", outcome)))
        .await?;

    Ok(())
}
