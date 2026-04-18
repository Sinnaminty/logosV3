//! `/profile set` subcommands — customise profile fields.
//!
//! - [`bio`] — set your profile bio text.
//! - [`banner`] — set a banner image URL (or attachment).
//! - [`colorway`] — set a custom accent colour for your profile embed.
//! - [`title`] — equip one of your owned catalog titles.
//! - [`customtitle`] — set a user-written title (requires the unlock).

use crate::pawthos::{
    consts::{MAX_ACTIVE_BADGES, MAX_CUSTOM_TITLE_LEN},
    enums::inventory_errors::InventoryError,
    enums::profile_errors::ProfileError,
    structs::shop_catalog::{self, BANNERS, COLORWAYS, TITLES},
    types::{Context, Result},
};
use crate::utils;
use poise::serenity_prelude::{self as serenity, AutocompleteChoice};

/// Profile customisation subcommands.
#[poise::command(
    slash_command,
    subcommands(
        "bio",
        "banner",
        "namedbanner",
        "colorway",
        "namedcolorway",
        "title",
        "customtitle",
        "badges"
    )
)]
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
/// Requires the Custom Banner Unlock from `/shop buy unlock`. You can provide
/// a URL or upload a file attachment (attachment takes priority).
#[poise::command(slash_command)]
pub async fn banner(
    ctx: Context<'_>,
    #[description = "Banner image URL"] url: Option<String>,
    #[description = "Banner image attachment (overrides URL)"] attachment: Option<
        serenity::Attachment,
    >,
) -> Result {
    let user_id = ctx.author().id;

    // Gate: requires the unlock.
    let unlocked = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| Ok(inv.unlocked_custom_banner))
        .await
        .unwrap_or(false);
    if !unlocked {
        return Err(InventoryError::FeatureLocked("banner").into());
    }

    let banner_url = attachment.as_ref().map(|a| a.url.clone()).or(url);

    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.banner_url = banner_url.clone();
            // Custom beats named: clear any named-banner equip so the render picks up the new custom.
            if banner_url.is_some() {
                p.active_banner_id = None;
            }
            Ok(())
        })
        .await?;

    let msg = if banner_url.is_some() {
        "Your banner has been updated!"
    } else {
        "Your banner has been cleared."
    };

    ctx.send(utils::reply_ok("Profile Set Banner", msg)).await?;
    Ok(())
}

/// Equip one of your owned named banners.
#[poise::command(slash_command)]
pub async fn namedbanner(
    ctx: Context<'_>,
    #[description = "Which banner to equip"]
    #[autocomplete = "owned_banners_ac"]
    id: String,
) -> Result {
    let user_id = ctx.author().id;
    let def = shop_catalog::lookup_banner(&id)
        .ok_or_else(|| InventoryError::UnknownItem(id.clone()))?;

    let owned = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| Ok(inv.owned_banners.iter().any(|b| b == &id)))
        .await
        .unwrap_or(false);
    if !owned {
        return Err(InventoryError::NotOwned(def.item.name.to_string()).into());
    }

    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.active_banner_id = Some(id.clone());
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Profile Set Named Banner",
        format!("Equipped banner: **{}**.", def.item.name),
    ))
    .await?;
    Ok(())
}

/// Set a custom accent colour for your profile card embed.
///
/// Requires the Custom Colorway Unlock from `/shop buy unlock`. Accepts bare
/// hex (`FF8800`) or `0x`-prefixed (`0xFF8800`).
#[poise::command(slash_command)]
pub async fn colorway(
    ctx: Context<'_>,
    #[description = "Hex colour code (e.g. FF8800 or 0xFF8800)"] color: String,
) -> Result {
    let user_id = ctx.author().id;

    // Gate: requires the unlock.
    let unlocked = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| Ok(inv.unlocked_custom_colorway))
        .await
        .unwrap_or(false);
    if !unlocked {
        return Err(InventoryError::FeatureLocked("colorway").into());
    }

    let trimmed = color.strip_prefix("0x").unwrap_or(&color);
    let color_int =
        u32::from_str_radix(trimmed, 16).map_err(|_| ProfileError::InvalidColorway)?;

    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.colorway = Some(color_int);
            // Custom beats named: clear named-equip so the render picks up the new custom.
            p.active_colorway_id = None;
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

/// Equip one of your owned named colorways.
#[poise::command(slash_command)]
pub async fn namedcolorway(
    ctx: Context<'_>,
    #[description = "Which colorway to equip"]
    #[autocomplete = "owned_colorways_ac"]
    id: String,
) -> Result {
    let user_id = ctx.author().id;
    let def = shop_catalog::lookup_colorway(&id)
        .ok_or_else(|| InventoryError::UnknownItem(id.clone()))?;

    let owned = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| {
            Ok(inv.owned_colorways.iter().any(|c| c == &id))
        })
        .await
        .unwrap_or(false);
    if !owned {
        return Err(InventoryError::NotOwned(def.item.name.to_string()).into());
    }

    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.active_colorway_id = Some(id.clone());
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Profile Set Named Colorway",
        format!(
            "Equipped colorway: **{}** (`#{:06X}`).",
            def.item.name, def.hex
        ),
    ))
    .await?;
    Ok(())
}

/// Equip one of your owned catalog titles.
///
/// Autocomplete shows only titles you've purchased. Use `/shop buy title <id>`
/// first to acquire one.
#[poise::command(slash_command)]
pub async fn title(
    ctx: Context<'_>,
    #[description = "Which title to equip"]
    #[autocomplete = "owned_titles_ac"]
    id: String,
) -> Result {
    let user_id = ctx.author().id;

    let def = shop_catalog::lookup_title(&id)
        .ok_or_else(|| InventoryError::UnknownItem(id.clone()))?;

    // Verify ownership.
    let owned = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| Ok(inv.owned_titles.iter().any(|t| t == &id)))
        .await
        .unwrap_or(false);
    if !owned {
        return Err(InventoryError::NotOwned(def.item.name.to_string()).into());
    }

    // Equip: clear custom flag, set active catalog title.
    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.active_title_id = Some(id.clone());
            p.use_custom_title = false;
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Profile Set Title",
        format!("Equipped title: **{}**.", def.item.name),
    ))
    .await?;
    Ok(())
}

/// Write your own title — requires the Custom Title Unlock from the shop.
#[poise::command(slash_command)]
pub async fn customtitle(
    ctx: Context<'_>,
    #[description = "Your custom title text"] text: String,
) -> Result {
    let user_id = ctx.author().id;
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return Err(InventoryError::CustomTooLong {
            field: "title",
            max: MAX_CUSTOM_TITLE_LEN,
        }
        .into());
    }
    if trimmed.chars().count() > MAX_CUSTOM_TITLE_LEN {
        return Err(InventoryError::CustomTooLong {
            field: "title",
            max: MAX_CUSTOM_TITLE_LEN,
        }
        .into());
    }

    // Must have the unlock.
    let unlocked = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| Ok(inv.unlocked_custom_title))
        .await
        .unwrap_or(false);
    if !unlocked {
        return Err(InventoryError::FeatureLocked("title").into());
    }

    // Store + auto-equip.
    ctx.data()
        .with_inventory_user_write(user_id, |inv| {
            inv.custom_title = Some(trimmed.to_string());
            Ok(())
        })
        .await?;
    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.use_custom_title = true;
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Profile Set Custom Title",
        format!("Your custom title is now: **{trimmed}**."),
    ))
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Autocomplete helpers
// ---------------------------------------------------------------------------

async fn owned_titles_ac(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let p = partial.to_lowercase();
    let owned = ctx
        .data()
        .with_inventory_user_read(ctx.author().id, |inv| Ok(inv.owned_titles.clone()))
        .await
        .unwrap_or_default();

    TITLES
        .iter()
        .filter(|t| owned.iter().any(|id| id == t.item.id))
        .filter(|t| {
            t.item.name.to_lowercase().contains(&p) || t.item.id.to_lowercase().contains(&p)
        })
        .take(25)
        .map(|t| AutocompleteChoice::new(t.item.name.to_string(), t.item.id.to_string()))
        .collect()
}

async fn owned_colorways_ac(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let p = partial.to_lowercase();
    let owned = ctx
        .data()
        .with_inventory_user_read(ctx.author().id, |inv| Ok(inv.owned_colorways.clone()))
        .await
        .unwrap_or_default();

    COLORWAYS
        .iter()
        .filter(|c| owned.iter().any(|id| id == c.item.id))
        .filter(|c| {
            c.item.name.to_lowercase().contains(&p) || c.item.id.to_lowercase().contains(&p)
        })
        .take(25)
        .map(|c| AutocompleteChoice::new(c.item.name.to_string(), c.item.id.to_string()))
        .collect()
}

async fn owned_banners_ac(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let p = partial.to_lowercase();
    let owned = ctx
        .data()
        .with_inventory_user_read(ctx.author().id, |inv| Ok(inv.owned_banners.clone()))
        .await
        .unwrap_or_default();

    BANNERS
        .iter()
        .filter(|b| owned.iter().any(|id| id == b.item.id))
        .filter(|b| {
            b.item.name.to_lowercase().contains(&p) || b.item.id.to_lowercase().contains(&p)
        })
        .take(25)
        .map(|b| AutocompleteChoice::new(b.item.name.to_string(), b.item.id.to_string()))
        .collect()
}

// ---------------------------------------------------------------------------
// Badges — pin up to MAX_ACTIVE_BADGES to your profile card
// ---------------------------------------------------------------------------

/// Pin up to 3 owned badges to your profile card.
///
/// Pass each badge in order (slot1 first, slot2 second, slot3 third). Omit
/// a slot to leave it empty. Badges are shared across lootbox pulls and
/// achievement unlocks — autocomplete lists every badge you currently own.
/// Running this command with no args clears your pinned badges.
#[poise::command(slash_command)]
pub async fn badges(
    ctx: Context<'_>,
    #[description = "First badge slot"]
    #[autocomplete = "owned_badges_ac"]
    slot1: Option<String>,
    #[description = "Second badge slot"]
    #[autocomplete = "owned_badges_ac"]
    slot2: Option<String>,
    #[description = "Third badge slot"]
    #[autocomplete = "owned_badges_ac"]
    slot3: Option<String>,
) -> Result {
    let user_id = ctx.author().id;
    let raw: Vec<String> = [slot1, slot2, slot3].into_iter().flatten().collect();

    // Dedupe while preserving first-seen order.
    let mut ids: Vec<String> = Vec::new();
    for id in raw {
        if !ids.iter().any(|x| x == &id) {
            ids.push(id);
        }
    }

    if ids.len() > MAX_ACTIVE_BADGES {
        return Err(InventoryError::TooManyBadges {
            max: MAX_ACTIVE_BADGES,
            attempted: ids.len(),
        }
        .into());
    }

    // Verify ownership of every ID — reject unknown or unowned entries before
    // mutating so the user either gets the full equip or a clean failure.
    let owned = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| Ok(inv.owned_badges.clone()))
        .await
        .unwrap_or_default();
    for id in &ids {
        if !owned.iter().any(|b| b == id) {
            let label = shop_catalog::resolve_badge_display(id)
                .map(|(_, n)| n.to_string())
                .unwrap_or_else(|| id.clone());
            return Err(InventoryError::NotOwned(label).into());
        }
    }

    ctx.data()
        .with_profile_user_write(user_id, |p| {
            p.active_badge_ids = ids.clone();
            Ok(())
        })
        .await?;

    let body = if ids.is_empty() {
        "Cleared your pinned badges.".to_string()
    } else {
        let rendered = ids
            .iter()
            .filter_map(|id| shop_catalog::resolve_badge_display(id).map(|(e, n)| format!("{e} {n}")))
            .collect::<Vec<_>>()
            .join(" · ");
        format!("Pinned: {rendered}")
    };

    ctx.send(utils::reply_ok("Profile Set Badges", body)).await?;
    Ok(())
}

/// Autocomplete over every badge the user owns, across lootbox and
/// achievement pools. Shows `{emoji} {name}` as the label.
async fn owned_badges_ac(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let p = partial.to_lowercase();
    let owned = ctx
        .data()
        .with_inventory_user_read(ctx.author().id, |inv| Ok(inv.owned_badges.clone()))
        .await
        .unwrap_or_default();

    owned
        .iter()
        .filter_map(|id| shop_catalog::resolve_badge_display(id).map(|(e, n)| (id, e, n)))
        .filter(|(id, _, name)| {
            name.to_lowercase().contains(&p) || id.to_lowercase().contains(&p)
        })
        .take(25)
        .map(|(id, emoji, name)| {
            AutocompleteChoice::new(format!("{emoji} {name}"), id.to_string())
        })
        .collect()
}
