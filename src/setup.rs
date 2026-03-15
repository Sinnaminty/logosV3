//! Bot startup helpers.
//!
//! This module re-exports the two primary setup functions so that `main.rs`
//! only needs a single `use crate::setup::*`-style import. It also owns the
//! [`APIKey`] deserialisation and the [`get_api_token`] helper that reads
//! the bot token from `s.json`.

pub use crate::framework::setup_framework;
pub use crate::logging::setup_logging;
pub use crate::pawthos::consts::INTENTS;
use crate::utils::ResultExt;
use poise::serenity_prelude as serenity;

/// Shape of the `s.json` secrets file.
///
/// ```json
/// { "token": "your-discord-bot-token-here" }
/// ```
///
/// Keep this file out of version control — it contains the bot's Discord token.
#[derive(serde::Deserialize)]
pub struct APIKey {
    /// The Discord bot token used to authenticate with the Gateway.
    pub token: String,
}

/// Read the Discord bot token from `s.json` in the working directory.
///
/// # Panics
/// Panics (via [`ResultExt::unwrap_or_log`]) if the file is missing or
/// contains invalid JSON. Both conditions are unrecoverable at startup.
pub fn get_api_token() -> String {
    let file_contents =
        std::fs::read_to_string("s.json").unwrap_or_log("setup::get_api_token::read_to_string");

    let api_key: APIKey =
        serenity::json::from_str(file_contents).unwrap_or_log("setup::json::from_str");
    api_key.token
}
