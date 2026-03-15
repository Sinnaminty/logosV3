//! `/mimic delete` subcommands — remove mimics and overrides.
//!
//! All three subcommands mutate the calling user's [`MimicUser`] record:
//!
//! - [`mimic`] — permanently remove a mimic from your list.
//! - [`active_mimic`] — un-set your active mimic (blocked if auto-mode is on).
//! - [`channel_override`] — remove the override for a specific channel.
//!
//! [`MimicUser`]: crate::pawthos::structs::mimic_user::MimicUser

use crate::pawthos::{
    enums::mimic_errors::MimicError,
    types::{Context, Result},
};
use crate::{commands::mimic::fetch_mimics, utils};
use poise::serenity_prelude::Channel;

/// Mimic deletion subcommands.
#[poise::command(
    slash_command,
    subcommands("mimic", "channel_override", "active_mimic")
)]
pub async fn delete(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Permanently delete one of your mimics.
///
/// Autocomplete lists your existing mimics. Deleting the active mimic does
/// **not** automatically unset `active_mimic` — you may want to run
/// `/mimic delete active_mimic` afterwards or set a new active mimic.
#[poise::command(slash_command)]
pub async fn mimic(ctx: Context<'_>, #[autocomplete = "fetch_mimics"] name: String) -> Result {
    let user_id = ctx.author().id;
    let target = name.trim();

    let deleted_mimic_name = ctx
        .data()
        .with_mimic_user_write(user_id, |user| {
            let idx = user
                .mimics
                .iter()
                .position(|m| m.name == target)
                .ok_or(MimicError::MimicNotFound)?;

            let removed = user.mimics.remove(idx);
            Ok(removed.name)
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Mimic Delete Mimic",
        format!("You deleted \"{}\"!", deleted_mimic_name),
    ))
    .await?;
    Ok(())
}

/// Remove the mimic override for a specific channel.
///
/// After this, messages in that channel will use your active mimic instead.
#[poise::command(slash_command)]
pub async fn channel_override(ctx: Context<'_>, channel: Channel) -> Result {
    let user_id = ctx.author().id;
    let channel_id = channel.id();

    let mimic_name = ctx
        .data()
        .with_mimic_user_write(user_id, |user| {
            let m = user
                .channel_override
                .remove(&channel_id)
                .ok_or(MimicError::NoChannelOverride)?;

            Ok(m.name)
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Mimic Delete channel_override",
        format!(
            "Successfully deleted {}'s channel override for channel {}",
            mimic_name, channel
        ),
    ))
    .await?;
    Ok(())
}

/// Unset your active mimic.
///
/// Fails if auto-mode is currently enabled, because disabling the active
/// mimic with auto-mode on would leave the bot in an inconsistent state.
/// Disable auto-mode first (`/mimic set auto Disable`) before running this.
#[poise::command(slash_command)]
pub async fn active_mimic(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    let mimic_name = ctx
        .data()
        .with_mimic_user_write(user_id, |user| {
            if user.auto_mode {
                return Err(MimicError::DeleteActiveMimicWithAutoModeEnabled);
            }

            let m = user.active_mimic.take().ok_or(MimicError::NoActiveMimic)?;
            Ok(m.name)
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Mimic Delete active_mimic",
        format!("Successfully deleted your active_mimic: {}", mimic_name),
    ))
    .await?;
    Ok(())
}
