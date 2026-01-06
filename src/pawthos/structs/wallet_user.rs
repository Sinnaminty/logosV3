use poise::serenity_prelude::Role;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletUser {
    #[serde(default)]
    pub tabs: i64,

    /// Unix timestamp (seconds) of last /daily claim (or 0 if never)
    #[serde(default)]
    pub last_daily_ts: i64,

    #[serde(default)]
    pub owned_roles: Vec<Role>,
}
