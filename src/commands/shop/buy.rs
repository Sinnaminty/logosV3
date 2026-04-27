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
        LOOTBOX_CHANCE_UNCOMMON, LOOTBOX_COST, LOOTBOX_SALVAGE, TAB_EMOJI,
    },
    enums::inventory_errors::InventoryError,
    structs::shop_catalog::{
        self, BadgeDef, COLORWAYS, LOOTBOX_POOL, Rarity, TITLES, UNLOCKS,
    },
    types::{Context, Result},
};
use crate::utils;
use poise::serenity_prelude::AutocompleteChoice;
use rand::Rng;

/// Shop purchase subcommands.
#[poise::command(
    slash_command,
    subcommands("title", "colorway", "unlock", "lootbox")
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
        Rarity::Achievement => "Achievement",
    }
}

fn odds_for(r: Rarity) -> f64 {
    match r {
        Rarity::Common => LOOTBOX_CHANCE_COMMON,
        Rarity::Uncommon => LOOTBOX_CHANCE_UNCOMMON,
        Rarity::Rare => LOOTBOX_CHANCE_RARE,
        Rarity::Legendary => LOOTBOX_CHANCE_LEGENDARY,
        Rarity::Achievement => 0.0,
    }
}
