//! `/profile` command suite — user profile cards with badges, banners, and colorways.
//!
//! # Commands in this file
//! - [`profile`] — parent command (required by Poise).
//! - [`view`] — display a user's profile card as a rich embed.
//!
//! # Sub-modules
//! - [`set`] — subcommands for customising profile fields.

use crate::commands::profile::set::*;
use crate::pawthos::{
    consts::{LOGOS_GREEN, TAB_EMOJI},
    enums::embed_type::EmbedType,
    types::{Context, Result},
};
use crate::utils;
use poise::serenity_prelude::{self as serenity, Color};
mod set;

/// Profile card commands — view and customise your profile.
#[poise::command(slash_command, subcommands("view", "set"))]
pub async fn profile(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Display a user's profile card as a rich embed.
///
/// Shows their bio, badges, tab balance, custom colorway, and banner.
/// Defaults to showing your own profile; pass a user to view theirs.
#[poise::command(slash_command)]
pub async fn view(
    ctx: Context<'_>,
    #[description = "User to view (defaults to yourself)"] user: Option<serenity::User>,
) -> Result {
    let target = user.as_ref().unwrap_or_else(|| ctx.author());
    let target_id = target.id;

    // Read profile data (bio, badges, banner, colorway).
    let profile = ctx
        .data()
        .with_profile_user_read(target_id, |p| Ok(p.clone()))
        .await
        .unwrap_or_default();

    // Read tab balance (may not exist for new users).
    let tabs = ctx
        .data()
        .with_wallet_user_read(target_id, |w| Ok(w.tabs))
        .await
        .unwrap_or(0);

    let accent = profile
        .colorway
        .map(Color::new)
        .unwrap_or(LOGOS_GREEN);

    let bio = profile
        .bio
        .as_deref()
        .unwrap_or("*No bio set. Use `/profile set bio` to add one!*");

    let badge_display = if profile.badges.is_empty() {
        "None".to_string()
    } else {
        profile
            .badges
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    };

    let display_name = target
        .global_name
        .as_deref()
        .unwrap_or(&target.name);

    let mut embed = utils::create_embed_builder(
        format!("{display_name}'s Profile"),
        bio,
        EmbedType::Neutral,
    )
    .color(accent)
    .thumbnail(
        target
            .avatar_url()
            .unwrap_or_else(|| target.default_avatar_url()),
    )
    .field("Badges", &badge_display, true)
    .field("Balance", format!("{tabs} {TAB_EMOJI}"), true);

    if let Some(banner) = &profile.banner_url {
        embed = embed.image(banner);
    }

    ctx.send(poise::CreateReply::default().embed(embed))
        .await?;
    Ok(())
}
