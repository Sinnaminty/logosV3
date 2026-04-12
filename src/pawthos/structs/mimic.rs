//! A single mimic persona.

use serde::{Deserialize, Serialize};

/// A named persona used by the mimic feature.
///
/// When a user talks as a mimic, a Discord webhook posts their message
/// with `name` as the username and `avatar_url` (if set) as the avatar,
/// making the message appear to come from a different identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mimic {
    /// The display name the webhook will use when posting as this mimic.
    pub name: String,

    /// Optional avatar URL for the mimic's webhook posts.
    ///
    /// `None` means the webhook uses its own default avatar. Can be set from
    /// a URL or from a file attachment at creation time via `/mimic add`.
    pub avatar_url: Option<String>,
}
