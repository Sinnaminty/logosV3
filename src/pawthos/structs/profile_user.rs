//! Per-user state for the profile card feature.

use crate::pawthos::structs::badge::Badge;
use serde::{Deserialize, Serialize};

/// All profile-related state for a single user.
///
/// Displayed as a rich embed via `/profile view`. Each field is optional so
/// new users start with a blank profile that fills in as they customise it.
/// The data model is designed for future expansion: badges, banners, and
/// colorways can be purchased from a shop.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileUser {
    /// A short bio or description set by the user.
    #[serde(default)]
    pub bio: Option<String>,

    /// Badges the user owns, displayed on their profile card.
    #[serde(default)]
    pub badges: Vec<Badge>,

    /// URL of a custom banner image shown at the bottom of the profile embed.
    #[serde(default)]
    pub banner_url: Option<String>,

    /// Custom accent colour (as a raw RGB u32) for the profile embed border.
    /// Falls back to the bot's default green when `None`.
    #[serde(default)]
    pub colorway: Option<u32>,
}
