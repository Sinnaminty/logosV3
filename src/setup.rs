use poise::serenity_prelude as serenity;

pub use crate::framework::setup_framework;
pub use crate::logging::setup_logging;
pub use crate::types::INTENTS;

use crate::utils::ResultExt;

#[derive(serde::Deserialize)]
pub struct APIKey {
    pub token: String,
}

pub fn get_api_token() -> String {
    let file_contents = std::fs::read_to_string("s.json").unwrap_or_log();
    let api_key: APIKey = serenity::json::from_str(file_contents).unwrap_or_log();
    api_key.token
}
