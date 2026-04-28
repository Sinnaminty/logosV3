//! `/profile` command suite — user profile cards with badges, banners, and colorways.
//!
//! # Commands in this file
//! - [`profile`] — parent command (required by Poise).
//! - [`view`] — display a user's profile card as a rich embed.
//!
//! # Sub-modules
//! - [`set`] — subcommands for customising profile fields.
//! - [`unset`] — subcommands for clearing equipped items.

use crate::commands::profile::set::set;
use crate::commands::profile::unset::unset;
use crate::pawthos::{
    consts::{LOGOS_GREEN, TAB_EMOJI},
    enums::embed_type::EmbedType,
    structs::inventory_user::InventoryUser,
    structs::profile_user::ProfileUser,
    structs::shop_catalog,
    types::{Context, Result},
};
use crate::utils;
use poise::serenity_prelude::{self as serenity, Color};
mod set;
mod unset;

/// Profile card commands — view and customise your profile.
#[poise::command(slash_command, subcommands("view", "set", "unset"))]
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

    // Read profile data (bio, badges, banner, colorway, equipped items).
    let profile = ctx
        .data()
        .with_profile_user_read(target_id, |p| Ok(p.clone()))
        .await
        .unwrap_or_default();

    // Read inventory (custom title + owned items).
    let inventory = ctx
        .data()
        .with_inventory_user_read(target_id, |i| Ok(i.clone()))
        .await
        .unwrap_or_default();

    // Read tab balance (may not exist for new users).
    let tabs = ctx
        .data()
        .with_wallet_user_read(target_id, |w| Ok(w.tabs))
        .await
        .unwrap_or(0);

    let accent = resolve_colorway(&profile);

    let bio = profile
        .bio
        .as_deref()
        .unwrap_or("*No bio set. Use `/profile set bio` to add one!*");

    let title_line = resolve_title(&profile, &inventory);
    let description = match title_line {
        Some(ref t) => format!("*✨ {t}*\n\n{bio}"),
        None => bio.to_string(),
    };

    let badge_display = render_active_badges(&profile, &inventory);

    let display_name = target
        .global_name
        .as_deref()
        .unwrap_or(&target.name);

    let mut embed = utils::create_embed_builder(
        format!("{display_name}'s Profile"),
        description,
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

    if let Some(banner) = resolve_banner(&profile) {
        embed = embed.image(banner);
    }

    ctx.send(poise::CreateReply::default().embed(embed))
        .await?;
    Ok(())
}

/// Resolve which title string (if any) to display on a profile card.
///
/// See [`ProfileUser`] doc for resolution priority.
fn resolve_title(profile: &ProfileUser, inventory: &InventoryUser) -> Option<String> {
    if profile.use_custom_title {
        inventory.custom_title.clone()
    } else if let Some(ref id) = profile.active_title_id {
        shop_catalog::lookup_title(id).map(|t| t.item.name.to_string())
    } else {
        None
    }
}

/// Resolve which accent colour to render.
///
/// Priority: named colorway (catalog lookup) → custom hex → default.
fn resolve_colorway(profile: &ProfileUser) -> Color {
    if let Some(ref id) = profile.active_colorway_id
        && let Some(def) = shop_catalog::lookup_colorway(id)
    {
        return Color::new(def.hex);
    }
    profile.colorway.map(Color::new).unwrap_or(LOGOS_GREEN)
}

/// Resolve which banner URL (if any) to render.
fn resolve_banner(profile: &ProfileUser) -> Option<String> {
    profile.banner_url.clone()
}

/// Render the Badges field on `/profile view`.
///
/// Iterates `active_badge_ids` (the pinned slots), resolving each via
/// [`shop_catalog::resolve_badge_display`]. IDs the user no longer owns or
/// that no longer match a catalog entry are silently dropped. Returns
/// `"None"` if the user has nothing to show.
fn render_active_badges(profile: &ProfileUser, inventory: &InventoryUser) -> String {
    let rendered: Vec<String> = profile
        .active_badge_ids
        .iter()
        .filter(|id| inventory.owned_badges.iter().any(|o| o == *id))
        .filter_map(|id| shop_catalog::resolve_badge_display(id).map(|(e, n)| format!("{e} {n}")))
        .collect();
    if rendered.is_empty() {
        "None".to_string()
    } else {
        rendered.join(" · ")
    }
}
