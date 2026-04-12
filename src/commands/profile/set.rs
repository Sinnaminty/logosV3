//! `/profile set` subcommands — customise profile fields.
//!
//! - [`bio`] — set your profile bio text.
//! - [`banner`] — set a banner image URL (or attachment).
//! - [`colorway`] — set a custom accent colour for your profile embed.

use crate::pawthos::{
    enums::profile_errors::ProfileError,
    types::{Context, Result},
};
use crate::utils;
use poise::serenity_prelude as serenity;

/// Profile customisation subcommands.
#[poise::command(slash_command, subcommands("bio", "banner", "colorway"))]
pub async fn set(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Set your profile bio (shown on your profile card).
#[poise::command(slash_command)]
pub async fn bio(
    ctx: Context<'_>,
    #[description = "Your new bio text"] text: String,
) -> Result {
    let user_id = ctx.author().id;

    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.bio = Some(text.clone());
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Profile Set Bio",
        format!("Your bio has been updated to:\n> {text}"),
    ))
    .await?;
    Ok(())
}

/// Set a custom banner image for your profile card.
///
/// You can provide a URL or upload a file attachment (attachment takes priority).
#[poise::command(slash_command)]
pub async fn banner(
    ctx: Context<'_>,
    #[description = "Banner image URL"] url: Option<String>,
    #[description = "Banner image attachment (overrides URL)"] attachment: Option<
        serenity::Attachment,
    >,
) -> Result {
    let user_id = ctx.author().id;
    let banner_url = attachment
        .as_ref()
        .map(|a| a.url.clone())
        .or(url);

    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.banner_url = banner_url.clone();
            Ok(())
        })
        .await?;

    let msg = if banner_url.is_some() {
        "Your banner has been updated!"
    } else {
        "Your banner has been cleared."
    };

    ctx.send(utils::reply_ok("Profile Set Banner", msg))
        .await?;
    Ok(())
}

/// Set a custom accent colour for your profile card embed.
///
/// Accepts bare hex (`FF8800`) or `0x`-prefixed (`0xFF8800`).
#[poise::command(slash_command)]
pub async fn colorway(
    ctx: Context<'_>,
    #[description = "Hex colour code (e.g. FF8800 or 0xFF8800)"] color: String,
) -> Result {
    let user_id = ctx.author().id;
    let trimmed = color.strip_prefix("0x").unwrap_or(&color);
    let color_int =
        u32::from_str_radix(trimmed, 16).map_err(|_| ProfileError::InvalidColorway)?;

    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.colorway = Some(color_int);
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Profile Set Colorway",
        format!("Your profile accent colour is now `#{trimmed}`!"),
    ))
    .await?;
    Ok(())
}
