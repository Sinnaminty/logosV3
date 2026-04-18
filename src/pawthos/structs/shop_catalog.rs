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

/// Lootbox pull pool — Phase 8 fills this in. Keep empty until then so the
/// lootbox UI remains hidden (or, once the command exists, refuses to run).
pub const LOOTBOX_POOL: &[BadgeDef] = &[];

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
