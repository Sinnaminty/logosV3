pub use crate::framework::setup_framework;
pub use crate::logging::setup_logging;
pub use crate::pawthos::consts::INTENTS;
use crate::utils::ResultExt;
use poise::serenity_prelude as serenity;

#[derive(serde::Deserialize)]
pub struct APIKey {
    pub token: String,
}

pub fn get_api_token() -> String {
    let file_contents =
        std::fs::read_to_string("s.json").unwrap_or_log("setup::get_api_token::read_to_string");

    let api_key: APIKey =
        serenity::json::from_str(file_contents).unwrap_or_log("setup::json::from_str");
    api_key.token
}
