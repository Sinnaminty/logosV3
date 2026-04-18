//! `/profile set` subcommands — customise profile fields.
//!
//! - [`bio`] — set your profile bio text.
//! - [`banner`] — set a banner image URL (or attachment).
//! - [`colorway`] — set a custom accent colour for your profile embed.
//! - [`title`] — equip one of your owned catalog titles.
//! - [`customtitle`] — set a user-written title (requires the unlock).

use crate::pawthos::{
    consts::MAX_CUSTOM_TITLE_LEN,
    enums::inventory_errors::InventoryError,
    enums::profile_errors::ProfileError,
    structs::shop_catalog::{self, TITLES},
    types::{Context, Result},
};
use crate::utils;
use poise::serenity_prelude::{self as serenity, AutocompleteChoice};

/// Profile customisation subcommands.
#[poise::command(
    slash_command,
    subcommands("bio", "banner", "colorway", "title", "customtitle")
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
