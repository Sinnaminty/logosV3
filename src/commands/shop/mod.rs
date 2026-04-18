//! `/shop` command suite — browse the catalog, view your inventory, purchase items.
//!
//! # Commands in this module
//! - [`shop`] — parent command (required by Poise).
//! - [`browse`] — list every catalog item, grouped by category.
//! - [`inventory`] — show what the calling user owns.
//!
//! # Sub-modules
//! - [`buy`] — purchase subcommands (title, unlock, …).

use crate::commands::shop::buy::buy;
use crate::commands::shop::gift::gift;
use crate::pawthos::{
    consts::TAB_EMOJI,
    enums::embed_type::EmbedType,
    structs::inventory_user::InventoryUser,
    structs::shop_catalog::{self, BANNERS, COLORWAYS, LOOTBOX_ITEM, LOOTBOX_POOL, TITLES, UNLOCKS},
    types::{Context, Result},
};
use crate::utils;
mod buy;
mod gift;

/// Shop commands — browse cosmetics, purchase items, gift to others, view your inventory.
#[poise::command(slash_command, subcommands("browse", "inventory", "buy", "gift"))]
pub async fn shop(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Browse every item in the shop, grouped by category.
///
/// Items are listed by ID (copy-paste friendly), display name, cost, and
/// a short description. Categories with nothing defined are hidden.
#[poise::command(slash_command)]
pub async fn browse(ctx: Context<'_>) -> Result {
    let mut description = String::new();

    if !TITLES.is_empty() {
        description.push_str("**✨ Titles** — a line under your name on `/profile view`\n");
        for t in TITLES {
            description.push_str(&format!(
                "`{}` — **{}** · {} {TAB_EMOJI} — *{}*\n",
                t.item.id, t.item.name, t.item.cost, t.item.description,
            ));
        }
        description.push('\n');
    }

    if !COLORWAYS.is_empty() {
        description.push_str("**🎨 Colorways** — your profile embed's accent colour\n");
        for c in COLORWAYS {
            description.push_str(&format!(
                "`{}` — **{}** · {} {TAB_EMOJI}\n",
                c.item.id, c.item.name, c.item.cost,
            ));
        }
        description.push('\n');
    }

    if !BANNERS.is_empty() {
        description.push_str("**🖼️ Banners** — the image at the bottom of your profile card\n");
        for b in BANNERS {
            description.push_str(&format!(
                "`{}` — **{}** · {} {TAB_EMOJI}\n",
                b.item.id, b.item.name, b.item.cost,
            ));
        }
        description.push('\n');
    }

    if !LOOTBOX_POOL.is_empty() {
        use crate::pawthos::consts::{
            LOOTBOX_CHANCE_COMMON, LOOTBOX_CHANCE_LEGENDARY, LOOTBOX_CHANCE_RARE,
            LOOTBOX_CHANCE_UNCOMMON, LOOTBOX_SALVAGE,
        };
        use crate::pawthos::structs::shop_catalog::Rarity;
        let count_of = |r: Rarity| -> usize {
            LOOTBOX_POOL.iter().filter(|b| b.item.rarity == r).count()
        };
        description.push_str(&format!(
            "**🎁 Badge Lootbox** — `/shop buy lootbox` · {} {TAB_EMOJI} per pull\n\
             *{}*\n\
             Duplicates salvage for **{LOOTBOX_SALVAGE} {TAB_EMOJI}**.\n\n\
             **Odds:**\n\
             🟢 Common {:.0}% — {} items\n\
             🔵 Uncommon {:.0}% — {} items\n\
             🟣 Rare {:.0}% — {} items\n\
             🟡 Legendary {:.0}% — {} items\n\n",
            LOOTBOX_ITEM.cost,
            LOOTBOX_ITEM.description,
            LOOTBOX_CHANCE_COMMON * 100.0, count_of(Rarity::Common),
            LOOTBOX_CHANCE_UNCOMMON * 100.0, count_of(Rarity::Uncommon),
            LOOTBOX_CHANCE_RARE * 100.0, count_of(Rarity::Rare),
            LOOTBOX_CHANCE_LEGENDARY * 100.0, count_of(Rarity::Legendary),
        ));
    }

    if !UNLOCKS.is_empty() {
        description.push_str("**🔓 Unlocks** — enable custom `/profile set …` commands\n");
        for u in UNLOCKS {
            description.push_str(&format!(
                "`{}` — **{}** · {} {TAB_EMOJI} — *{}*\n",
                u.id, u.name, u.cost, u.description,
            ));
        }
    }

    if description.is_empty() {
        description.push_str("*The shop is empty right now. Check back soon!*");
    }

    let embed = utils::create_embed_builder("Shop", description, EmbedType::Neutral);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Show what you own: titles, colorways, banners, badges, unlocks.
///
/// Response is ephemeral — only you can see it.
#[poise::command(slash_command)]
pub async fn inventory(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    let inv = ctx
        .data()
        .with_inventory_user_read(user_id, |i| Ok(i.clone()))
        .await
        .unwrap_or_default();

    let embed = utils::create_embed_builder(
        "Your Inventory",
        render_summary(&inv),
        EmbedType::Neutral,
    )
    .field("Titles", render_titles(&inv), false)
    .field("Colorways", render_colorways(&inv), false)
    .field("Banners", render_banners(&inv), false)
    .field("Badges", render_badges(&inv), false)
    .field("Unlocks", render_unlocks(&inv), false);

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Inventory rendering helpers
// ---------------------------------------------------------------------------

fn render_summary(inv: &InventoryUser) -> String {
    let badge_count = inv.owned_badges.len();
    let unlock_count = [
        inv.unlocked_custom_title,
        inv.unlocked_custom_colorway,
        inv.unlocked_custom_banner,
    ]
    .iter()
    .filter(|b| **b)
    .count();
    format!(
        "**{}** titles · **{}** colorways · **{}** banners · **{}** badges · **{}** unlocks",
        inv.owned_titles.len(),
        inv.owned_colorways.len(),
        inv.owned_banners.len(),
        badge_count,
        unlock_count,
    )
}

fn render_titles(inv: &InventoryUser) -> String {
    if inv.owned_titles.is_empty() && inv.custom_title.is_none() {
        return "*None owned.*".into();
    }
    let mut lines: Vec<String> = inv
        .owned_titles
        .iter()
        .map(|id| match shop_catalog::lookup_title(id) {
            Some(t) => format!("• **{}**", t.item.name),
            None => format!("• `{id}` *(unknown)*"),
        })
        .collect();
    if let Some(custom) = &inv.custom_title {
        lines.push(format!("• **{custom}** *(custom)*"));
    }
    lines.join("\n")
}

fn render_colorways(inv: &InventoryUser) -> String {
    if inv.owned_colorways.is_empty() {
        return "*None owned.*".into();
    }
    inv.owned_colorways
        .iter()
        .map(|id| match shop_catalog::lookup_colorway(id) {
            Some(c) => format!("• **{}** · `#{:06X}`", c.item.name, c.hex),
            None => format!("• `{id}` *(unknown)*"),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_banners(inv: &InventoryUser) -> String {
    if inv.owned_banners.is_empty() {
        return "*None owned.*".into();
    }
    inv.owned_banners
        .iter()
        .map(|id| match shop_catalog::lookup_banner(id) {
            Some(b) => format!("• **{}**", b.item.name),
            None => format!("• `{id}` *(unknown)*"),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_badges(inv: &InventoryUser) -> String {
    if inv.owned_badges.is_empty() {
        return "*None owned.*".into();
    }
    let (lootbox, achievement): (Vec<_>, Vec<_>) = inv
        .owned_badges
        .iter()
        .partition(|id| id.starts_with("box_"));

    let mut sections: Vec<String> = Vec::new();
    if !lootbox.is_empty() {
        let line = lootbox
            .iter()
            .map(|id| match shop_catalog::lookup_badge(id) {
                Some(b) => format!("{} {}", b.emoji, b.item.name),
                None => format!("`{id}`"),
            })
            .collect::<Vec<_>>()
            .join(" · ");
        sections.push(format!("*Lootbox:* {line}"));
    }
    if !achievement.is_empty() {
        let line = achievement
            .iter()
            .map(|id| format!("`{id}`"))
            .collect::<Vec<_>>()
            .join(" · ");
        sections.push(format!("*Achievements:* {line}"));
    }
    sections.join("\n")
}

fn render_unlocks(inv: &InventoryUser) -> String {
    let mut lines: Vec<&str> = Vec::new();
    if inv.unlocked_custom_title {
        lines.push("• Custom Title");
    }
    if inv.unlocked_custom_colorway {
        lines.push("• Custom Colorway");
    }
    if inv.unlocked_custom_banner {
        lines.push("• Custom Banner");
    }
    if lines.is_empty() {
        "*None.*".into()
    } else {
        lines.join("\n")
    }
}
