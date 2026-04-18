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
    consts::TAB_EMOJI,
    enums::inventory_errors::InventoryError,
    structs::shop_catalog::{self, TITLES, UNLOCKS},
    types::{Context, Result},
};
use crate::utils;
use poise::serenity_prelude::AutocompleteChoice;

/// Shop purchase subcommands.
#[poise::command(slash_command, subcommands("title", "unlock"))]
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
    Ok(())
}

/// Buy a one-time unlock (custom title, colorway, or banner).
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
                "unlock_custom_colorway" => inv.unlocked_custom_colorway,
                "unlock_custom_banner" => inv.unlocked_custom_banner,
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
            match id.as_str() {
                "unlock_custom_title" => inv.unlocked_custom_title = true,
                "unlock_custom_colorway" => inv.unlocked_custom_colorway = true,
                "unlock_custom_banner" => inv.unlocked_custom_banner = true,
                _ => {}
            }
            inv.tabs_spent_lifetime += item.cost;
            Ok(())
        })
        .await?;

    let next_step = match id.as_str() {
        "unlock_custom_title" => "Set one with `/profile set customtitle <text>`.",
        "unlock_custom_colorway" => "Set one with `/profile set colorway <hex>`.",
        "unlock_custom_banner" => "Set one with `/profile set banner <url|attachment>`.",
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
