//! `/shop gift …` subcommands.
//!
//! Sending a gift is a *direct purchase for someone else* — the sender pays
//! the item's cost plus a small [`GIFT_FEE`] and the item is added to the
//! recipient's inventory, even if they've never interacted with the bot
//! before.
//!
//! Categories are separate subcommands so each can have its own autocomplete
//! scoped to the right table.
//!
//! # Flow
//!
//! 1. Reject self-gifts.
//! 2. Reject if the recipient already owns the item.
//! 3. Charge sender (item cost + fee).
//! 4. Grant to recipient + increment `gifts_received`.
//! 5. Update sender stats (`gifts_sent`, `tabs_spent_lifetime`).
//! 6. Post an in-channel announcement (public; sender/recipient mentioned).

use crate::pawthos::{
    consts::{GIFT_FEE, TAB_EMOJI},
    enums::inventory_errors::InventoryError,
    structs::shop_catalog::{self, COLORWAYS, TITLES},
    types::{Context, Result},
};
use poise::serenity_prelude::{self as serenity, AutocompleteChoice};

/// Gift a shop item to another user.
#[poise::command(slash_command, subcommands("title", "colorway"))]
pub async fn gift(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Gift a catalog title to another user.
#[poise::command(slash_command)]
pub async fn title(
    ctx: Context<'_>,
    #[description = "Who to gift"] recipient: serenity::User,
    #[description = "Which title"]
    #[autocomplete = "giftable_titles"]
    id: String,
) -> Result {
    let def = shop_catalog::lookup_title(&id)
        .ok_or_else(|| InventoryError::UnknownItem(id.clone()))?;

    let gift_context = GiftContext {
        sender: ctx.author(),
        recipient: &recipient,
        item_id: id.clone(),
        item_name: def.item.name.to_string(),
        item_cost: def.item.cost,
        category_label: "Title",
    };

    perform_gift(ctx, gift_context, |inv, id| {
        inv.owned_titles.iter().any(|t| t == id)
    }, |inv, id| {
        inv.owned_titles.push(id.to_string());
    })
    .await
}

/// Gift a named colorway to another user.
#[poise::command(slash_command)]
pub async fn colorway(
    ctx: Context<'_>,
    #[description = "Who to gift"] recipient: serenity::User,
    #[description = "Which colorway"]
    #[autocomplete = "giftable_colorways"]
    id: String,
) -> Result {
    let def = shop_catalog::lookup_colorway(&id)
        .ok_or_else(|| InventoryError::UnknownItem(id.clone()))?;

    let gift_context = GiftContext {
        sender: ctx.author(),
        recipient: &recipient,
        item_id: id.clone(),
        item_name: def.item.name.to_string(),
        item_cost: def.item.cost,
        category_label: "Colorway",
    };

    perform_gift(ctx, gift_context, |inv, id| {
        inv.owned_colorways.iter().any(|c| c == id)
    }, |inv, id| {
        inv.owned_colorways.push(id.to_string());
    })
    .await
}

// ---------------------------------------------------------------------------
// Shared gift flow
// ---------------------------------------------------------------------------

/// Captures everything `perform_gift` needs that doesn't depend on item type.
struct GiftContext<'a> {
    sender: &'a serenity::User,
    recipient: &'a serenity::User,
    item_id: String,
    item_name: String,
    item_cost: i64,
    category_label: &'static str,
}

/// Run the full gift sequence. `already_owns` / `grant` encapsulate the
/// category-specific vec manipulation.
async fn perform_gift(
    ctx: Context<'_>,
    gc: GiftContext<'_>,
    already_owns: impl Fn(
        &crate::pawthos::structs::inventory_user::InventoryUser,
        &str,
    ) -> bool,
    grant: impl Fn(&mut crate::pawthos::structs::inventory_user::InventoryUser, &str),
) -> Result {
    if gc.sender.id == gc.recipient.id {
        return Err(InventoryError::GiftToSelf.into());
    }

    let recipient_owns = ctx
        .data()
        .with_inventory_user_read(gc.recipient.id, |inv| Ok(already_owns(inv, &gc.item_id)))
        .await
        .unwrap_or(false);
    if recipient_owns {
        return Err(InventoryError::RecipientOwns(
            gc.recipient.name.clone(),
            gc.item_name.clone(),
        )
        .into());
    }

    let total = gc.item_cost + GIFT_FEE;

    // 1. Charge sender — propagates NotEnoughTabs.
    ctx.data()
        .with_wallet_user_write(gc.sender.id, |w| w.remove_tabs(total))
        .await?;

    // 2. Grant to recipient.
    ctx.data()
        .with_inventory_user_write(gc.recipient.id, |inv| {
            grant(inv, &gc.item_id);
            inv.gifts_received = inv.gifts_received.saturating_add(1);
            Ok(())
        })
        .await?;

    // 3. Sender-side stats.
    ctx.data()
        .with_inventory_user_write(gc.sender.id, |inv| {
            inv.gifts_sent = inv.gifts_sent.saturating_add(1);
            inv.tabs_spent_lifetime = inv.tabs_spent_lifetime.saturating_add(total);
            Ok(())
        })
        .await?;

    // 4. Announce publicly in the invoking channel.
    let sender_id = gc.sender.id;
    let recipient_id = gc.recipient.id;
    let announce = format!(
        "🎁 <@{sender_id}> gifted **{}** ({}) to <@{recipient_id}> for **{} {TAB_EMOJI}** (includes **{} {TAB_EMOJI}** fee).",
        gc.item_name, gc.category_label, total, GIFT_FEE,
    );
    ctx.send(
        poise::CreateReply::default()
            .content(announce)
            .allowed_mentions(
                serenity::CreateAllowedMentions::default()
                    .users(vec![sender_id, recipient_id])
                    .everyone(false)
                    .all_roles(false),
            ),
    )
    .await?;

    // 5. Achievement checks for both sides — in the same channel.
    let channel = ctx.channel_id();
    let http = ctx.http();
    ctx.data().check_achievements(sender_id, channel, http).await;
    ctx.data()
        .check_achievements(recipient_id, channel, http)
        .await;

    Ok(())
}

// ---------------------------------------------------------------------------
// Autocomplete — all gifters see the full catalog (you can gift anything).
// ---------------------------------------------------------------------------

async fn giftable_titles(_ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
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

async fn giftable_colorways(_ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
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
