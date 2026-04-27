//! Per-user state for the shop / inventory feature.
//!
//! Holds owned cosmetic items, interaction statistics, and unlock flags for
//! paywalled customisation options. Items are referenced by string ID into
//! the static [`super::shop_catalog`] registry so the on-disk schema is
//! resilient to catalog reshuffles.

use serde::{Deserialize, Serialize};

/// All inventory-related state for a single user.
///
/// Owned IDs reference entries in [`crate::pawthos::structs::shop_catalog`].
/// Unlock flags enable paywalled `/profile set …` commands — see
/// [`crate::pawthos::structs::profile_user`] for how the rendering pipeline
/// resolves equipped items against these flags.
///
/// Every field uses `#[serde(default)]` so old `user.json` snapshots without
/// an inventory field deserialise cleanly into a blank default.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryUser {
    // ---------------------------------------------------------------------
    // Owned item IDs
    // ---------------------------------------------------------------------
    /// Title IDs the user has purchased. Equipped via `active_title_id` on
    /// [`crate::pawthos::structs::profile_user::ProfileUser`].
    #[serde(default)]
    pub owned_titles: Vec<String>,

    /// Named colorway IDs the user has purchased. Owners can freely swap
    /// between any owned colorway via `/profile set namedcolorway`.
    #[serde(default)]
    pub owned_colorways: Vec<String>,

    /// Badge IDs the user has earned — combined lootbox pool (`box_*`)
    /// and achievement grants (`ach_*`).
    #[serde(default)]
    pub owned_badges: Vec<String>,

    // ---------------------------------------------------------------------
    // Custom-field slots
    // ---------------------------------------------------------------------
    /// User-supplied title text, capped at
    /// [`crate::pawthos::consts::MAX_CUSTOM_TITLE_LEN`] chars. Gated by
    /// [`Self::unlocked_custom_title`] (one-time unlock).
    #[serde(default)]
    pub custom_title: Option<String>,

    /// Gates `/profile set customtitle <text>`. Custom colorway and custom
    /// banner are *not* unlocks — they charge per-set in the command
    /// handler instead.
    #[serde(default)]
    pub unlocked_custom_title: bool,

    // ---------------------------------------------------------------------
    // Interaction statistics (feed the achievement system in Phase 7)
    // ---------------------------------------------------------------------
    /// Incremented in the message handler on every guild message from this
    /// user. Drives message-count achievements.
    #[serde(default)]
    pub messages_sent: u64,

    /// Incremented each time this user's `/shop gift` purchase succeeds.
    #[serde(default)]
    pub gifts_sent: u32,

    /// Incremented each time this user receives a gift from another user.
    #[serde(default)]
    pub gifts_received: u32,

    /// Lifetime count of `/shop buy lootbox` invocations that actually pulled
    /// a new badge (excludes duplicate-salvage outcomes? — see Phase 8).
    #[serde(default)]
    pub lootboxes_opened: u32,

    /// Count of successful faucet-bounty claims.
    #[serde(default)]
    pub faucet_claims: u32,

    /// Running total of tabs spent across all shop purchases and gift fees.
    #[serde(default)]
    pub tabs_spent_lifetime: i64,

    // ---------------------------------------------------------------------
    // Achievement progress
    // ---------------------------------------------------------------------
    /// Achievement IDs the user has already unlocked. Used to de-duplicate
    /// announcements when the check runs.
    #[serde(default)]
    pub unlocked_achievements: Vec<String>,
}
