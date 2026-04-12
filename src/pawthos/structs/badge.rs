//! A purchasable profile badge.

use serde::{Deserialize, Serialize};

/// A badge displayed on a user's profile card.
///
/// Badges are cosmetic items that can be purchased from the shop (future)
/// or awarded for achievements. They show up as emoji + name pairs in the
/// profile embed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Badge {
    /// The display name of the badge.
    pub name: String,

    /// A Discord emoji string (custom or unicode) shown alongside the name.
    pub emoji: String,
}

impl std::fmt::Display for Badge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.emoji, self.name)
    }
}
