//! `/shop buy …` subcommands.
//!
//! Each subcommand performs a three-step purchase:
//!
//! 1. **Ownership check** via `with_inventory_user_read`. Returning
//!    [`InventoryError::AlreadyOwned`] short-circuits before any tab
//!    deduction.
//! 2. **Tab deduction** via `with_wallet_user_write`. Surfaces
//!    [`crate::pawthos::enums::wallet_errors::WalletError::NotEnoughTabs`]
//!    on insufficient balance.
//! 3. **Grant + stats update** via `with_inventory_user_write`.
//!
//! Steps 2 and 3 are not atomic across the two sub-struct writes, but both
//! operate on the same in-memory `UserDB` so the window is vanishingly small.
//! See `SHOP_PLAN.md` § "Locking discipline" for the long-term mitigation.

use crate::pawthos::{
    consts::{
        LOOTBOX_CHANCE_COMMON, LOOTBOX_CHANCE_LEGENDARY, LOOTBOX_CHANCE_RARE,
        LOOTBOX_CHANCE_UNCOMMON, LOOTBOX_COST, LOOTBOX_SALVAGE, ROLE_COLOR_COST,
        ROLE_NAME_COST, TAB_EMOJI,
    },
    enums::color_errors::ColorError,
    enums::inventory_errors::InventoryError,
    structs::shop_catalog::{
        self, BadgeDef, COLORWAYS, LOOTBOX_POOL, Rarity, TITLES, UNLOCKS,
    },
    types::{Context, Result},
};
use crate::utils;
use poise::serenity_prelude::{self as serenity, AutocompleteChoice, EditRole};
use rand::Rng;

/// Shop purchase subcommands.
///
/// `rolecolor` and `rolename` are per-use cosmetics — they don't grant an
/// inventory item, they spend tabs to apply a change to your custom colour
/// role on the current guild. Each call charges separately.
#[poise::command(
    slash_command,
    subcommands("title", "colorway", "unlock", "lootbox", "rolecolor", "rolename")
)]
pub async fn buy(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Buy a catalog title. Use `/profile set title <id>` afterwards to equip it.
#[poise::command(slash_command)]
pub async fn title(
    ctx: Context<'_>,
    #[description = "Which title to buy"]
    #[autocomplete = "buyable_titles"]
    id: String,
) -> Result {
    let def = shop_catalog::lookup_title(&id)
        .ok_or_else(|| InventoryError::UnknownItem(id.clone()))?;

    let user_id = ctx.author().id;

    // 1. Ownership check. `NoUserFound` means no inventory yet → not owned.
    let already = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| {
            Ok(inv.owned_titles.iter().any(|t| t == &id))
        })
        .await
        .unwrap_or(false);
    if already {
        return Err(InventoryError::AlreadyOwned(def.item.name.to_string()).into());
    }

    // 2. Charge. Propagates `WalletError::NotEnoughTabs` to the error handler.
    ctx.data()
        .with_wallet_user_write(user_id, |w| w.remove_tabs(def.item.cost))
        .await?;

    // 3. Grant.
    ctx.data()
        .with_inventory_user_write(user_id, |inv| {
            inv.owned_titles.push(id.clone());
            inv.tabs_spent_lifetime += def.item.cost;
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Shop Buy Title",
        format!(
            "You bought **{}** for **{} {TAB_EMOJI}**!\nEquip it with `/profile set title {}`.",
            def.item.name, def.item.cost, def.item.id,
        ),
    ))
    .await?;

    ctx.data()
        .check_achievements(user_id, ctx.channel_id(), ctx.http())
        .await;
    Ok(())
}

/// Buy a named colorway. Equip with `/profile set namedcolorway <id>`.
#[poise::command(slash_command)]
pub async fn colorway(
    ctx: Context<'_>,
    #[description = "Which colorway to buy"]
    #[autocomplete = "buyable_colorways"]
    id: String,
) -> Result {
    let def = shop_catalog::lookup_colorway(&id)
        .ok_or_else(|| InventoryError::UnknownItem(id.clone()))?;

    let user_id = ctx.author().id;

    let already = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| {
            Ok(inv.owned_colorways.iter().any(|c| c == &id))
        })
        .await
        .unwrap_or(false);
    if already {
        return Err(InventoryError::AlreadyOwned(def.item.name.to_string()).into());
    }

    ctx.data()
        .with_wallet_user_write(user_id, |w| w.remove_tabs(def.item.cost))
        .await?;

    ctx.data()
        .with_inventory_user_write(user_id, |inv| {
            inv.owned_colorways.push(id.clone());
            inv.tabs_spent_lifetime += def.item.cost;
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Shop Buy Colorway",
        format!(
            "You bought **{}** (`#{:06X}`) for **{} {TAB_EMOJI}**!\nEquip it with `/profile set namedcolorway {}`.",
            def.item.name, def.hex, def.item.cost, def.item.id,
        ),
    ))
    .await?;

    ctx.data()
        .check_achievements(user_id, ctx.channel_id(), ctx.http())
        .await;
    Ok(())
}

/// Buy a one-time unlock. Currently only the custom-title unlock exists.
#[poise::command(slash_command)]
pub async fn unlock(
    ctx: Context<'_>,
    #[description = "Which unlock to buy"]
    #[autocomplete = "buyable_unlocks"]
    id: String,
) -> Result {
    let item = UNLOCKS
        .iter()
        .find(|u| u.id == id)
        .ok_or_else(|| InventoryError::UnknownItem(id.clone()))?;

    let user_id = ctx.author().id;

    let already = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| {
            Ok(match id.as_str() {
                "unlock_custom_title" => inv.unlocked_custom_title,
                _ => false,
            })
        })
        .await
        .unwrap_or(false);
    if already {
        return Err(InventoryError::AlreadyOwned(item.name.to_string()).into());
    }

    ctx.data()
        .with_wallet_user_write(user_id, |w| w.remove_tabs(item.cost))
        .await?;

    ctx.data()
        .with_inventory_user_write(user_id, |inv| {
            if id.as_str() == "unlock_custom_title" {
                inv.unlocked_custom_title = true;
            }
            inv.tabs_spent_lifetime += item.cost;
            Ok(())
        })
        .await?;

    let next_step = match id.as_str() {
        "unlock_custom_title" => "Set one with `/profile set customtitle <text>`.",
        _ => "",
    };

    ctx.send(utils::reply_ok(
        "Shop Buy Unlock",
        format!(
            "You unlocked **{}** for **{} {TAB_EMOJI}**!\n{}",
            item.name, item.cost, next_step,
        ),
    ))
    .await?;

    ctx.data()
        .check_achievements(user_id, ctx.channel_id(), ctx.http())
        .await;
    Ok(())
}

// ---------------------------------------------------------------------------
// Autocomplete helpers
// ---------------------------------------------------------------------------

async fn buyable_titles(_ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let p = partial.to_lowercase();
    TITLES
        .iter()
        .filter(|t| {
            t.item.name.to_lowercase().contains(&p) || t.item.id.to_lowercase().contains(&p)
        })
        .take(25)
        .map(|t| AutocompleteChoice::new(t.item.name.to_string(), t.item.id.to_string()))
        .collect()
}

async fn buyable_unlocks(_ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let p = partial.to_lowercase();
    UNLOCKS
        .iter()
        .filter(|u| u.name.to_lowercase().contains(&p) || u.id.to_lowercase().contains(&p))
        .take(25)
        .map(|u| AutocompleteChoice::new(u.name.to_string(), u.id.to_string()))
        .collect()
}

async fn buyable_colorways(_ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let p = partial.to_lowercase();
    COLORWAYS
        .iter()
        .filter(|c| {
            c.item.name.to_lowercase().contains(&p) || c.item.id.to_lowercase().contains(&p)
        })
        .take(25)
        .map(|c| AutocompleteChoice::new(c.item.name.to_string(), c.item.id.to_string()))
        .collect()
}

// ---------------------------------------------------------------------------
// Lootbox (Phase 8)
// ---------------------------------------------------------------------------

/// Roll a badge lootbox. Duplicates salvage for tabs.
///
/// Rolls a rarity tier using the `LOOTBOX_CHANCE_*` constants, then picks a
/// random badge of that rarity from [`LOOTBOX_POOL`]. If the user already
/// owns that badge, they get [`LOOTBOX_SALVAGE`] tabs back instead.
#[poise::command(slash_command)]
pub async fn lootbox(ctx: Context<'_>) -> Result {
    if LOOTBOX_POOL.is_empty() {
        return Err(
            InventoryError::UnknownItem("lootbox pool is currently empty".into()).into(),
        );
    }

    let user_id = ctx.author().id;

    // 1. Charge up front.
    ctx.data()
        .with_wallet_user_write(user_id, |w| w.remove_tabs(LOOTBOX_COST))
        .await?;

    // 2. Roll rarity + pick badge.
    let pull = {
        let mut rng = rand::thread_rng();
        let rarity = roll_rarity(&mut rng);
        // Fallback: if a rarity happens to have no candidates, degrade to
        // Common. Keeps the flow robust against lopsided pool edits.
        let candidates: Vec<&BadgeDef> = LOOTBOX_POOL
            .iter()
            .filter(|b| b.item.rarity == rarity)
            .collect();
        let candidates = if candidates.is_empty() {
            LOOTBOX_POOL
                .iter()
                .filter(|b| b.item.rarity == Rarity::Common)
                .collect::<Vec<_>>()
        } else {
            candidates
        };
        let idx = rng.gen_range(0..candidates.len());
        *candidates[idx]
    };

    // 3. Dup check.
    let already_owned = ctx
        .data()
        .with_inventory_user_read(user_id, |inv| {
            Ok(inv.owned_badges.iter().any(|b| b == pull.item.id))
        })
        .await
        .unwrap_or(false);

    // 4. Stats + grant (or salvage).
    ctx.data()
        .with_inventory_user_write(user_id, |inv| {
            inv.lootboxes_opened = inv.lootboxes_opened.saturating_add(1);
            inv.tabs_spent_lifetime = inv.tabs_spent_lifetime.saturating_add(LOOTBOX_COST);
            if !already_owned {
                inv.owned_badges.push(pull.item.id.to_string());
            }
            Ok(())
        })
        .await?;

    if already_owned {
        ctx.data()
            .with_wallet_user_write(user_id, |w| {
                w.add_tabs(LOOTBOX_SALVAGE);
                Ok(())
            })
            .await?;
    }

    let message = if already_owned {
        format!(
            "🔁 **Duplicate!** You rolled {} **{}** ({}). Salvaged for **{LOOTBOX_SALVAGE} {TAB_EMOJI}**.",
            pull.emoji,
            pull.item.name,
            rarity_name(pull.item.rarity),
        )
    } else {
        format!(
            "🎉 You pulled {} **{}** — *{}*\n**{}** rarity · {}% chance",
            pull.emoji,
            pull.item.name,
            pull.item.description,
            rarity_name(pull.item.rarity),
            (odds_for(pull.item.rarity) * 100.0) as u32,
        )
    };

    ctx.send(utils::reply_ok("Lootbox", message)).await?;

    ctx.data()
        .check_achievements(user_id, ctx.channel_id(), ctx.http())
        .await;
    Ok(())
}

/// Weighted rarity roll using [`LOOTBOX_CHANCE_*`] constants.
///
/// Orders checks from rarest to most common so the cumulative probability
/// comparisons work against a single uniform `[0, 1)` sample.
fn roll_rarity(rng: &mut impl Rng) -> Rarity {
    let r: f64 = rng.r#gen();
    let mut threshold = 0.0;

    threshold += LOOTBOX_CHANCE_LEGENDARY;
    if r < threshold {
        return Rarity::Legendary;
    }
    threshold += LOOTBOX_CHANCE_RARE;
    if r < threshold {
        return Rarity::Rare;
    }
    threshold += LOOTBOX_CHANCE_UNCOMMON;
    if r < threshold {
        return Rarity::Uncommon;
    }
    Rarity::Common
}

fn rarity_name(r: Rarity) -> &'static str {
    match r {
        Rarity::Common => "Common",
        Rarity::Uncommon => "Uncommon",
        Rarity::Rare => "Rare",
        Rarity::Legendary => "Legendary",
    }
}

fn odds_for(r: Rarity) -> f64 {
    match r {
        Rarity::Common => LOOTBOX_CHANCE_COMMON,
        Rarity::Uncommon => LOOTBOX_CHANCE_UNCOMMON,
        Rarity::Rare => LOOTBOX_CHANCE_RARE,
        Rarity::Legendary => LOOTBOX_CHANCE_LEGENDARY,
    }
}

// ---------------------------------------------------------------------------
// Per-use role cosmetics
// ---------------------------------------------------------------------------
//
// Custom colour roles are identified by a leading zero-width space (`\u{200B}`)
// in their name, which keeps them distinct from normal server roles. Both
// commands find the user's existing colour role via that prefix; if no such
// role exists, they create one using a sensible default for the field they
// don't touch (display name on rolecolor, no colour on rolename).
//
// Discord API work happens **before** any tab charge so a permission failure
// or rate-limit on Discord's side never costs the user tabs.

/// Change the colour of your custom colour role. Charged
/// [`ROLE_COLOR_COST`] tabs every call.
///
/// Accepts bare hex (`FF8800`) or `0x`-prefixed (`0xFF8800`). If you don't
/// have a colour role yet, one is created for you (named after your display
/// name) and assigned. **Special case:** `#000000` is silently mapped to
/// `rgb(1, 1, 1)` because Discord renders role colour `0` as the default
/// text colour rather than black.
#[poise::command(slash_command, guild_only)]
pub async fn rolecolor(
    ctx: Context<'_>,
    #[description = "Hex code (e.g. FF8800 or 0xFF8800)"] color: String,
) -> Result {
    let user_id = ctx.author().id;
    let guild_id = ctx.guild_id().unwrap();

    // Validate before doing any I/O.
    let trimmed = color.strip_prefix("0x").unwrap_or(&color);
    let color_int =
        u32::from_str_radix(trimmed, 16).map_err(|_| ColorError::IncorrectFormat)?;
    let role_color = if color_int == 0 {
        serenity::Colour::from_rgb(1, 1, 1)
    } else {
        serenity::Colour::new(color_int)
    };

    // Find or create the user's colour role.
    let member = guild_id.member(ctx.http(), user_id).await?;
    let guild_roles = guild_id.roles(ctx.http()).await?;

    if let Some(mut r) = find_user_flair_role(&member, &guild_roles) {
        r.edit(ctx.http(), EditRole::new().colour(role_color)).await?;
    } else {
        let display = ctx
            .author()
            .global_name
            .clone()
            .unwrap_or_else(|| ctx.author().name.clone());
        let role_name = format!("\u{200B}{display}");
        let new_role = guild_id
            .create_role(ctx.http(), EditRole::new().colour(role_color).name(role_name))
            .await?;
        member.add_role(ctx.http(), new_role.id).await?;
    }

    // Charge after Discord API success.
    let tabs = ctx
        .data()
        .with_wallet_user_write(user_id, |w| w.remove_tabs(ROLE_COLOR_COST))
        .await?;
    ctx.data()
        .with_inventory_user_write(user_id, |inv| {
            inv.tabs_spent_lifetime = inv.tabs_spent_lifetime.saturating_add(ROLE_COLOR_COST);
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Shop Buy Role Color",
        format!(
            "Your role colour is now `#{trimmed}` for **{ROLE_COLOR_COST} {TAB_EMOJI}**. Balance: **{tabs} {TAB_EMOJI}**.",
        ),
    ))
    .await?;

    ctx.data()
        .check_achievements(user_id, ctx.channel_id(), ctx.http())
        .await;
    Ok(())
}

/// Rename your custom colour role. Charged [`ROLE_NAME_COST`] tabs every call.
///
/// If you don't have a colour role yet, one is created with the given name
/// and no colour (use `/shop buy rolecolor` afterwards to set one).
#[poise::command(slash_command, guild_only)]
pub async fn rolename(
    ctx: Context<'_>,
    #[description = "New name for your role"] name: String,
) -> Result {
    let user_id = ctx.author().id;
    let guild_id = ctx.guild_id().unwrap();

    // Zero-width-space prefix marks this as a managed colour role.
    let role_name = format!("\u{200B}{name}");

    // Find or create the user's colour role.
    let member = guild_id.member(ctx.http(), user_id).await?;
    let guild_roles = guild_id.roles(ctx.http()).await?;

    if let Some(mut r) = find_user_flair_role(&member, &guild_roles) {
        r.edit(ctx.http(), EditRole::new().name(&role_name)).await?;
    } else {
        let new_role = guild_id
            .create_role(ctx.http(), EditRole::new().name(&role_name))
            .await?;
        member.add_role(ctx.http(), new_role.id).await?;
    }

    // Charge after Discord API success.
    let tabs = ctx
        .data()
        .with_wallet_user_write(user_id, |w| w.remove_tabs(ROLE_NAME_COST))
        .await?;
    ctx.data()
        .with_inventory_user_write(user_id, |inv| {
            inv.tabs_spent_lifetime = inv.tabs_spent_lifetime.saturating_add(ROLE_NAME_COST);
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Shop Buy Role Name",
        format!(
            "Your role name is now **{name}** for **{ROLE_NAME_COST} {TAB_EMOJI}**. Balance: **{tabs} {TAB_EMOJI}**.",
        ),
    ))
    .await?;

    ctx.data()
        .check_achievements(user_id, ctx.channel_id(), ctx.http())
        .await;
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared role-resolution
// ---------------------------------------------------------------------------

/// Find the user's existing "flair" colour role on the current guild.
///
/// Preferred: a role whose name starts with `\u{200B}` (the marker convention
/// applied by `rolecolor` and `rolename` whenever they create or rename a
/// role). This is fast and unambiguous.
///
/// Fallback for pre-convention roles (created manually in the guild UI, or
/// by an older bot version that didn't use the marker): a *single*
/// non-managed, non-hoist, non-default-coloured role on the member. The
/// "single" guard is deliberate — when the user has multiple coloured roles
/// (e.g., a flair role plus a moderator role), there's no safe heuristic to
/// pick the right one, so we return `None` and let the caller create a
/// fresh role rather than silently mutate the wrong one.
fn find_user_flair_role(
    member: &serenity::Member,
    guild_roles: &std::collections::HashMap<serenity::RoleId, serenity::Role>,
) -> Option<serenity::Role> {
    if let Some(r) = member
        .roles
        .iter()
        .filter_map(|id| guild_roles.get(id))
        .filter(|r| r.name.starts_with('\u{200B}'))
        .cloned()
        .next_back()
    {
        return Some(r);
    }

    let mut candidates = member
        .roles
        .iter()
        .filter_map(|id| guild_roles.get(id))
        .filter(|r| !r.managed && !r.hoist && r.colour.0 != 0);
    let first = candidates.next()?.clone();
    if candidates.next().is_some() {
        return None;
    }
    Some(first)
}
