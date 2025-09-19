use crate::types::EmbedType;
use poise::serenity_prelude::{self as serenity};
/// this is a trait!!
pub trait ResultExt<T, E> {
    /// Unwraps the result, logging the error and panicking if it's an Err.
    fn unwrap_or_log(self) -> T;
}

impl<T, E: std::fmt::Display> ResultExt<T, E> for Result<T, E> {
    fn unwrap_or_log(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                log::error!("Unrecoverable error: {e}");
                panic!();
            }
        }
    }
}

pub fn create_embed_builder(
    title: impl Into<String>,
    description: impl Into<String>,
    embed_type: EmbedType,
) -> serenity::builder::CreateEmbed {
    serenity::builder::CreateEmbed::new()
        .title(title)
        .description(description)
        .timestamp(serenity::Timestamp::now())
        .footer(serenity::builder::CreateEmbedFooter::new(
            "Powered by caffeine and lambda functions.",
        ))
        .author(serenity::builder::CreateEmbedAuthor::new("Logos"))
        .color(embed_type.into_color())
}
