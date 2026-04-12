//! Embed colour categories used by the reply helpers in [`crate::utils`].

use crate::pawthos::consts::{LOGOS_GREEN, LOGOS_RED};
use poise::serenity_prelude::Color;

/// Semantic colour category for a Discord embed.
///
/// Pass one of these variants to [`crate::utils::create_embed_builder`] (or use
/// the higher-level [`crate::utils::reply_ok`] / [`crate::utils::reply_err`] /
/// [`crate::utils::reply_info`] shortcuts) to give every embed a consistent
/// accent colour.
pub enum EmbedType {
    /// A successful operation — renders with [`LOGOS_GREEN`].
    Good,
    /// A failed or error state — renders with [`LOGOS_RED`].
    Bad,
    /// Informational or neutral content — renders with Discord's fabled pink.
    Neutral,
}

impl EmbedType {
    /// Convert the variant into the corresponding [`Color`] value.
    pub fn into_color(self) -> Color {
        match self {
            EmbedType::Good => LOGOS_GREEN,
            EmbedType::Bad => LOGOS_RED,
            EmbedType::Neutral => Color::FABLED_PINK,
        }
    }
}
