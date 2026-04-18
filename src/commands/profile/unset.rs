//! `/profile unset` subcommands — clear equipped items without deleting ownership.

use crate::pawthos::types::{Context, Result};
use crate::utils;

/// Clear equipped items.
#[poise::command(slash_command, subcommands("title"))]
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
