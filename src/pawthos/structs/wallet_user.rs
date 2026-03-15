use poise::serenity_prelude::Role;
use serde::{Deserialize, Serialize};

use crate::pawthos::{consts::DAILY_REWARD, enums::wallet_errors::WalletError};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletUser {
    #[serde(default)]
    pub tabs: i64,

    #[serde(default)]
    pub owned_roles: Vec<Role>,
}

impl WalletUser {
    pub fn add_tabs(&mut self, tabs: i64) -> i64 {
        self.tabs += tabs;
        self.tabs
    }

    pub fn remove_tabs(&mut self, cost: i64) -> Result<i64, WalletError> {
        if self.tabs < cost {
            Err(WalletError::NotEnoughTabs {
                cost,
                balance: self.tabs,
            })
        } else {
            self.tabs -= cost;
            Ok(self.tabs)
        }
    }

    pub fn claim_daily(&mut self) -> i64 {
        self.add_tabs(DAILY_REWARD)
    }
}
