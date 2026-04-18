//! Static shop catalog.
//!
//! All purchasable items are defined as compile-time constants here. Adding or
//! removing an item is a code change — the on-disk user data only stores item
//! IDs, so catalog reshuffles cannot corrupt existing inventories.
//!
//! # Layout
//!
//! Items are split across typed tables so each category can carry the extra
//! data it needs (colorways carry hex values, banners carry URLs, lootbox
//! badges carry emoji strings). All tables share the common [`ShopItem`]
//! header, so [`lookup`] can find any item by ID regardless of category.
//!
//! # ID conventions
//!
//! - `title_*`         — titles
//! - `colorway_*`      — named colorways
//! - `banner_*`        — named banners
//! - `unlock_*`        — one-time paywall unlocks
//! - `box_*`           — lootbox-pool badges (Phase 8)
//! - `ach_*`           — achievement badges (Phase 7)
//!
//! Namespacing by prefix lets `/shop inventory` partition a user's
//! `owned_badges` vec into lootbox vs. achievement sections without storing
//! a separate tag.

use crate::pawthos::consts::{LOOTBOX_COST, MAX_CUSTOM_TITLE_LEN};
use crate::pawthos::structs::inventory_user::InventoryUser;
use crate::pawthos::structs::wallet_user::WalletUser;

/// Rarity tier of a shop item. Drives lootbox roll probabilities and
/// optional UI affordances (colour tags, sort order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
    /// Achievement-granted items. Not rollable from a lootbox.
    Achievement,
}

/// Which shop section an item belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Title,
    Colorway,
    Banner,
    Badge,
    /// One-time paywall unlock (custom title, custom colorway, custom banner).
    Unlock,
    /// Lootbox pull service — buying one rolls a random badge.
    Lootbox,
}

/// The common header every catalog entry carries.
///
/// Typed definition structs ([`TitleDef`], [`ColorwayDef`], etc.) embed this
/// plus their category-specific payload. [`lookup`] returns this header so
/// callers don't need to branch on category unless they need the payload.
#[derive(Debug, Clone, Copy)]
pub struct ShopItem {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i64,
    pub category: Category,
    pub rarity: Rarity,
}

#[derive(Debug, Clone, Copy)]
pub struct TitleDef {
    pub item: ShopItem,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorwayDef {
    pub item: ShopItem,
    /// 24-bit RGB colour value (`0xRRGGBB`).
    pub hex: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct BannerDef {
    pub item: ShopItem,
    /// Absolute image URL.
    pub url: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct BadgeDef {
    pub item: ShopItem,
    /// Discord emoji string — custom (`<:name:id>`) or unicode (`🔥`).
    pub emoji: &'static str,
}

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

/// Purchasable titles — 10 tabs each.
pub const TITLES: &[TitleDef] = &[
    TitleDef { item: ShopItem {
        id: "title_tab_hoarder", name: "Tab Hoarder",
        description: "For those who save every last tab.",
        cost: 10, category: Category::Title, rarity: Rarity::Common,
    }},
    TitleDef { item: ShopItem {
        id: "title_early_adopter", name: "Early Adopter",
        description: "Here before the merch drops.",
        cost: 10, category: Category::Title, rarity: Rarity::Common,
    }},
    TitleDef { item: ShopItem {
        id: "title_certified_gremlin", name: "Certified Gremlin",
        description: "Your behaviour is noted.",
        cost: 10, category: Category::Title, rarity: Rarity::Common,
    }},
    TitleDef { item: ShopItem {
        id: "title_caffeine_dependent", name: "Caffeine Dependent",
        description: "Powered by legal stimulants.",
        cost: 10, category: Category::Title, rarity: Rarity::Common,
    }},
    TitleDef { item: ShopItem {
        id: "title_professional_lurker", name: "Professional Lurker",
        description: "Reads everything. Says nothing.",
        cost: 10, category: Category::Title, rarity: Rarity::Common,
    }},
    TitleDef { item: ShopItem {
        id: "title_night_owl", name: "Night Owl",
        description: "Active when the sun isn't.",
        cost: 10, category: Category::Title, rarity: Rarity::Common,
    }},
    TitleDef { item: ShopItem {
        id: "title_early_bird", name: "Early Bird",
        description: "Up before the standups.",
        cost: 10, category: Category::Title, rarity: Rarity::Common,
    }},
    TitleDef { item: ShopItem {
        id: "title_keyboard_warrior", name: "Keyboard Warrior",
        description: "Typing for the cause.",
        cost: 10, category: Category::Title, rarity: Rarity::Common,
    }},
];

/// Named colorways — 20 tabs each. Paired with an RGB hex value.
pub const COLORWAYS: &[ColorwayDef] = &[
    ColorwayDef { item: ShopItem {
        id: "colorway_sunset", name: "Sunset",
        description: "Warm coral red.",
        cost: 20, category: Category::Colorway, rarity: Rarity::Common,
    }, hex: 0xFF6B6B },
    ColorwayDef { item: ShopItem {
        id: "colorway_ocean", name: "Ocean",
        description: "Deep blue.",
        cost: 20, category: Category::Colorway, rarity: Rarity::Common,
    }, hex: 0x4A90E2 },
    ColorwayDef { item: ShopItem {
        id: "colorway_neon_pink", name: "Neon Pink",
        description: "Loud and proud.",
        cost: 20, category: Category::Colorway, rarity: Rarity::Common,
    }, hex: 0xFF1493 },
    ColorwayDef { item: ShopItem {
        id: "colorway_midnight", name: "Midnight",
        description: "Almost black.",
        cost: 20, category: Category::Colorway, rarity: Rarity::Common,
    }, hex: 0x1A1A3E },
    ColorwayDef { item: ShopItem {
        id: "colorway_gold", name: "Gold",
        description: "Premium yellow.",
        cost: 20, category: Category::Colorway, rarity: Rarity::Common,
    }, hex: 0xFFD700 },
    ColorwayDef { item: ShopItem {
        id: "colorway_lavender", name: "Lavender",
        description: "Soft purple.",
        cost: 20, category: Category::Colorway, rarity: Rarity::Common,
    }, hex: 0xB57EDC },
    ColorwayDef { item: ShopItem {
        id: "colorway_crimson", name: "Crimson",
        description: "Classic red.",
        cost: 20, category: Category::Colorway, rarity: Rarity::Common,
    }, hex: 0xDC143C },
    ColorwayDef { item: ShopItem {
        id: "colorway_mint", name: "Mint",
        description: "Cool green.",
        cost: 20, category: Category::Colorway, rarity: Rarity::Common,
    }, hex: 0x98D8A1 },
];

/// Named banners — Phase 4 will fill this in once the image catalog and
/// hosting are finalised. Kept empty for now so `/shop browse` hides the
/// section until content exists.
pub const BANNERS: &[BannerDef] = &[];

/// Lootbox pull pool.
///
/// 10 badges total, partitioned by rarity so the distribution from
/// [`crate::pawthos::consts`]'s `LOOTBOX_CHANCE_*` probabilities has at least
/// one candidate per tier:
///
/// | Rarity | Count | Odds |
/// |---|---|---|
/// | Common    | 4 | 60% |
/// | Uncommon  | 3 | 25% |
/// | Rare      | 2 | 10% |
/// | Legendary | 1 | 5%  |
///
/// Uses unicode emojis so new servers don't need custom Discord emoji setup.
/// Grow the pool by appending new entries — the lootbox flow handles any size.
pub const LOOTBOX_POOL: &[BadgeDef] = &[
    // --- Common (60%) ----------------------------------------------------
    BadgeDef {
        item: ShopItem {
            id: "box_coffee", name: "Coffee Addict",
            description: "Fueled by caffeine.",
            cost: 0, category: Category::Badge, rarity: Rarity::Common,
        },
        emoji: "☕",
    },
    BadgeDef {
        item: ShopItem {
            id: "box_bookworm", name: "Bookworm",
            description: "Reads the docs.",
            cost: 0, category: Category::Badge, rarity: Rarity::Common,
        },
        emoji: "📖",
    },
    BadgeDef {
        item: ShopItem {
            id: "box_pixel_pusher", name: "Pixel Pusher",
            description: "Shipper of CSS.",
            cost: 0, category: Category::Badge, rarity: Rarity::Common,
        },
        emoji: "🖼️",
    },
    BadgeDef {
        item: ShopItem {
            id: "box_moonlit", name: "Moonlit",
            description: "Working past bedtime.",
            cost: 0, category: Category::Badge, rarity: Rarity::Common,
        },
        emoji: "🌙",
    },
    // --- Uncommon (25%) --------------------------------------------------
    BadgeDef {
        item: ShopItem {
            id: "box_speedrunner", name: "Speedrunner",
            description: "Beat the standup.",
            cost: 0, category: Category::Badge, rarity: Rarity::Uncommon,
        },
        emoji: "🚀",
    },
    BadgeDef {
        item: ShopItem {
            id: "box_trailblazer", name: "Trailblazer",
            description: "Commits before coffee.",
            cost: 0, category: Category::Badge, rarity: Rarity::Uncommon,
        },
        emoji: "🧭",
    },
    BadgeDef {
        item: ShopItem {
            id: "box_stargazer", name: "Stargazer",
            description: "Collects repo stars.",
            cost: 0, category: Category::Badge, rarity: Rarity::Uncommon,
        },
        emoji: "⭐",
    },
    // --- Rare (10%) ------------------------------------------------------
    BadgeDef {
        item: ShopItem {
            id: "box_alchemist", name: "Alchemist",
            description: "Turns bugs into features.",
            cost: 0, category: Category::Badge, rarity: Rarity::Rare,
        },
        emoji: "🧪",
    },
    BadgeDef {
        item: ShopItem {
            id: "box_code_wizard", name: "Code Wizard",
            description: "It's not magic, it's hashmaps.",
            cost: 0, category: Category::Badge, rarity: Rarity::Rare,
        },
        emoji: "🪄",
    },
    // --- Legendary (5%) --------------------------------------------------
    BadgeDef {
        item: ShopItem {
            id: "box_void_walker", name: "Void Walker",
            description: "Stared into the debugger and smiled.",
            cost: 0, category: Category::Badge, rarity: Rarity::Legendary,
        },
        emoji: "🌌",
    },
];

/// One-time paywall unlocks — 30 tabs each.
pub const UNLOCKS: &[ShopItem] = &[
    ShopItem {
        id: "unlock_custom_title", name: "Custom Title Unlock",
        description: "Enables `/profile set customtitle <text>` (up to 32 chars).",
        cost: 30, category: Category::Unlock, rarity: Rarity::Uncommon,
    },
    ShopItem {
        id: "unlock_custom_colorway", name: "Custom Colorway Unlock",
        description: "Enables `/profile set colorway <hex>` with any 24-bit colour.",
        cost: 30, category: Category::Unlock, rarity: Rarity::Uncommon,
    },
    ShopItem {
        id: "unlock_custom_banner", name: "Custom Banner Unlock",
        description: "Enables `/profile set banner <url|attachment>`.",
        cost: 30, category: Category::Unlock, rarity: Rarity::Uncommon,
    },
];

/// Virtual lootbox purchase — not backed by an entry in a typed table. The
/// lootbox command references this directly so `/shop browse` can list it
/// alongside real items.
pub const LOOTBOX_ITEM: ShopItem = ShopItem {
    id: "lootbox", name: "Badge Lootbox",
    description: "Rolls a random badge by rarity. Duplicates salvage for tabs.",
    cost: LOOTBOX_COST, category: Category::Lootbox, rarity: Rarity::Common,
};

// ---------------------------------------------------------------------------
// Lookup
// ---------------------------------------------------------------------------

/// Find any item across every table by its ID.
///
/// Linear scan — table sizes are small (<100 total) so this is fine.
pub fn lookup(id: &str) -> Option<&'static ShopItem> {
    TITLES.iter().map(|t| &t.item)
        .chain(COLORWAYS.iter().map(|c| &c.item))
        .chain(BANNERS.iter().map(|b| &b.item))
        .chain(LOOTBOX_POOL.iter().map(|b| &b.item))
        .chain(UNLOCKS.iter())
        .chain(std::iter::once(&LOOTBOX_ITEM))
        .find(|i| i.id == id)
}

/// Find a title definition by ID (includes rarity / cost / description).
pub fn lookup_title(id: &str) -> Option<&'static TitleDef> {
    TITLES.iter().find(|t| t.item.id == id)
}

/// Find a colorway by ID.
pub fn lookup_colorway(id: &str) -> Option<&'static ColorwayDef> {
    COLORWAYS.iter().find(|c| c.item.id == id)
}

/// Find a banner by ID.
pub fn lookup_banner(id: &str) -> Option<&'static BannerDef> {
    BANNERS.iter().find(|b| b.item.id == id)
}

/// Find a badge (from the lootbox pool) by ID.
pub fn lookup_badge(id: &str) -> Option<&'static BadgeDef> {
    LOOTBOX_POOL.iter().find(|b| b.item.id == id)
}

/// Pretty-print the custom-title max length for help messages.
pub fn custom_title_max_len() -> usize { MAX_CUSTOM_TITLE_LEN }

// ---------------------------------------------------------------------------
// Achievements (Phase 7)
// ---------------------------------------------------------------------------

/// A server-interaction achievement.
///
/// Unlocks are gated by a predicate that inspects the user's current
/// [`InventoryUser`] and [`WalletUser`]. On unlock the ID is pushed onto
/// `inventory.unlocked_achievements` and also into `inventory.owned_badges`
/// so it surfaces in `/shop inventory`'s Badges section (under
/// "Achievements" via the `ach_` prefix convention).
#[derive(Clone, Copy)]
pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub emoji: &'static str,
    pub description: &'static str,
    /// Predicate — `true` iff the user has earned this achievement.
    ///
    /// Pure; no I/O. Checked every time a stat mutates.
    pub check: fn(&InventoryUser, &WalletUser) -> bool,
}

/// Static achievement registry. ID prefix is `ach_` to distinguish from
/// lootbox badges (`box_`).
pub const ACHIEVEMENTS: &[Achievement] = &[
    Achievement {
        id: "ach_chatterbox",
        name: "Chatterbox",
        emoji: "💬",
        description: "Send 100 messages.",
        check: |i, _| i.messages_sent >= 100,
    },
    Achievement {
        id: "ach_lorekeeper",
        name: "Lorekeeper",
        emoji: "📚",
        description: "Send 1,000 messages.",
        check: |i, _| i.messages_sent >= 1_000,
    },
    Achievement {
        id: "ach_oracle",
        name: "Oracle",
        emoji: "🔮",
        description: "Send 10,000 messages.",
        check: |i, _| i.messages_sent >= 10_000,
    },
    Achievement {
        id: "ach_first_spend",
        name: "First Spend",
        emoji: "🎉",
        description: "Make your first shop purchase.",
        check: |i, _| i.tabs_spent_lifetime >= 1,
    },
    Achievement {
        id: "ach_spender",
        name: "Spender",
        emoji: "💸",
        description: "Spend 50 tabs across the shop.",
        check: |i, _| i.tabs_spent_lifetime >= 50,
    },
    Achievement {
        id: "ach_whale",
        name: "Whale",
        emoji: "🐋",
        description: "Spend 500 tabs across the shop.",
        check: |i, _| i.tabs_spent_lifetime >= 500,
    },
    Achievement {
        id: "ach_generous",
        name: "Generous",
        emoji: "🎁",
        description: "Send 1 gift.",
        check: |i, _| i.gifts_sent >= 1,
    },
    Achievement {
        id: "ach_philanthropist",
        name: "Philanthropist",
        emoji: "💝",
        description: "Send 10 gifts.",
        check: |i, _| i.gifts_sent >= 10,
    },
    Achievement {
        id: "ach_beloved",
        name: "Beloved",
        emoji: "🫂",
        description: "Receive 5 gifts.",
        check: |i, _| i.gifts_received >= 5,
    },
    Achievement {
        id: "ach_committed",
        name: "Committed",
        emoji: "🔥",
        description: "Hit a 7-day daily streak.",
        check: |_, w| w.current_streak >= 7,
    },
    Achievement {
        id: "ach_devoted",
        name: "Devoted",
        emoji: "🌟",
        description: "Hit a 30-day daily streak.",
        check: |_, w| w.current_streak >= 30,
    },
    Achievement {
        id: "ach_quick_fingers",
        name: "Quick Fingers",
        emoji: "⚡",
        description: "Claim 5 tab-reaction faucet drops.",
        check: |i, _| i.faucet_claims >= 5,
    },
    Achievement {
        id: "ach_treasure_hunter",
        name: "Treasure Hunter",
        emoji: "💎",
        description: "Open 10 lootboxes.",
        check: |i, _| i.lootboxes_opened >= 10,
    },
];

/// Find an achievement by ID.
pub fn lookup_achievement(id: &str) -> Option<&'static Achievement> {
    ACHIEVEMENTS.iter().find(|a| a.id == id)
}

/// Resolve a badge ID to its `(emoji, name)` for display.
///
/// Checks the lootbox pool first, then the achievement registry. Returns
/// `None` if the ID doesn't match either — callers should filter those out
/// when rendering so stale data in `owned_badges` doesn't break the view.
pub fn resolve_badge_display(id: &str) -> Option<(&'static str, &'static str)> {
    if let Some(b) = lookup_badge(id) {
        return Some((b.emoji, b.item.name));
    }
    if let Some(a) = lookup_achievement(id) {
        return Some((a.emoji, a.name));
    }
    None
}
