//! `/profile unset` subcommands — clear equipped items without deleting ownership.

use crate::pawthos::types::{Context, Result};
use crate::utils;

/// Clear equipped items.
#[poise::command(slash_command, subcommands("title", "colorway", "banner", "badges"))]
pub async fn unset(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Unequip your current title (catalog or custom). Ownership is preserved.
#[poise::command(slash_command)]
pub async fn title(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.active_title_id = None;
            p.use_custom_title = false;
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Profile Unset Title",
        "Your title has been cleared.",
    ))
    .await?;
    Ok(())
}

/// Reset your profile colorway to the default.
///
/// Clears both the named-equip and the custom hex so the embed falls back to
/// Logos green.
#[poise::command(slash_command)]
pub async fn colorway(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.active_colorway_id = None;
            p.colorway = None;
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Profile Unset Colorway",
        "Your profile colorway has been reset to the default.",
    ))
    .await?;
    Ok(())
}

/// Clear your profile banner.
#[poise::command(slash_command)]
pub async fn banner(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.banner_url = None;
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Profile Unset Banner",
        "Your profile banner has been cleared.",
    ))
    .await?;
    Ok(())
}

/// Clear all pinned profile badges. Ownership is preserved.
#[poise::command(slash_command)]
pub async fn badges(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.active_badge_ids.clear();
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Profile Unset Badges",
        "Your pinned badges have been cleared.",
    ))
    .await?;
    Ok(())
}
