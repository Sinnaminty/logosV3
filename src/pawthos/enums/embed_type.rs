use crate::pawthos::consts::{LOGOS_GREEN, LOGOS_RED};
use poise::serenity_prelude::Color;

pub enum EmbedType {
    Good,
    Bad,
    Neutral,
}

impl EmbedType {
    pub fn into_color(self) -> Color {
        match self {
            EmbedType::Good => LOGOS_GREEN,
            EmbedType::Bad => LOGOS_RED,
            EmbedType::Neutral => Color::FABLED_PINK,
        }
    }
}
