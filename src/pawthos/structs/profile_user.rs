//! Per-user state for the profile card feature.

use crate::pawthos::structs::badge::Badge;
use serde::{Deserialize, Serialize};

/// All profile-related state for a single user.
///
/// Displayed as a rich embed via `/profile view`. Each field is optional so
/// new users start with a blank profile that fills in as they customise it.
///
/// # Equipped-item pointers
///
/// `active_title_id` and `use_custom_title` decide what appears as the user's
/// title on their profile card. Resolution priority:
///
/// 1. If `use_custom_title` is true and
///    [`crate::pawthos::structs::inventory_user::InventoryUser::custom_title`]
///    is `Some`, render that.
/// 2. Else if `active_title_id` points at a catalog entry, render the
///    catalog title's display name.
/// 3. Else render no title.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileUser {
    /// A short bio or description set by the user.
    #[serde(default)]
    pub bio: Option<String>,

    /// Badges the user owns, displayed on their profile card.
    ///
    /// Legacy — Phase 2+ stores badge IDs on
    /// [`crate::pawthos::structs::inventory_user::InventoryUser::owned_badges`]
    /// instead. Kept here so old records deserialise; a future migration
    /// will drain this into the new storage.
    #[serde(default)]
    pub badges: Vec<Badge>,

    /// URL of a custom banner image shown at the bottom of the profile embed.
    ///
    /// Per-set charge: `/profile set banner <url|attachment>` deducts
    /// [`crate::pawthos::consts::BANNER_SET_COST`] tabs each time it stores
    /// a non-empty URL. Clearing the banner is free.
    #[serde(default)]
    pub banner_url: Option<String>,

    /// Custom accent colour (as a raw RGB u32) for the profile embed border.
    /// Falls back to the bot's default green when `None`.
    ///
    /// Per-set charge: `/profile set colorway <hex>` deducts
    /// [`crate::pawthos::consts::CUSTOM_COLORWAY_SET_COST`] tabs each time.
    /// Equipping a named colorway via `/profile set namedcolorway` from
    /// [`crate::pawthos::structs::inventory_user::InventoryUser::owned_colorways`]
    /// is free.
    #[serde(default)]
    pub colorway: Option<u32>,

    /// ID of the equipped catalog title, or `None` to display no title.
    #[serde(default)]
    pub active_title_id: Option<String>,

    /// When true, render
    /// [`crate::pawthos::structs::inventory_user::InventoryUser::custom_title`]
    /// instead of resolving `active_title_id`.
    #[serde(default)]
    pub use_custom_title: bool,

    /// ID of the equipped named colorway. Takes precedence over
    /// [`Self::colorway`] when rendering `/profile view`.
    #[serde(default)]
    pub active_colorway_id: Option<String>,

    /// Badge IDs pinned to the user's profile card, in display order.
    ///
    /// Capped at [`crate::pawthos::consts::MAX_ACTIVE_BADGES`] by the
    /// `/profile set badges` command. IDs may reference entries in either
    /// [`crate::pawthos::structs::shop_catalog::LOOTBOX_POOL`] (`box_*`
    /// prefix) or [`crate::pawthos::structs::shop_catalog::ACHIEVEMENTS`]
    /// (`ach_*` prefix). The resolver in `/profile view` checks both and
    /// skips any that don't match a live catalog entry.
    #[serde(default)]
    pub active_badge_ids: Vec<String>,
}
