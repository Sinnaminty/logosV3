//! Per-user state for the wallet/tab economy.

use poise::serenity_prelude::Role;
use serde::{Deserialize, Serialize};

use crate::pawthos::{consts::DAILY_REWARD, enums::wallet_errors::WalletError};

/// All wallet-related state for a single user.
///
/// "Tabs" are the in-server currency. Users earn them via `/daily` and spend
/// them on features like `/color set`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletUser {
    /// Current tab balance. Cannot go below zero (enforced by [`remove_tabs`]).
    ///
    /// `#[serde(default)]` means old JSON records without this field start at 0.
    ///
    /// [`remove_tabs`]: WalletUser::remove_tabs
    #[serde(default)]
    pub tabs: i64,

    /// Discord roles that the user purchased via `/color set`.
    ///
    /// Persisted so role ownership survives bot restarts (useful for future
    /// reconciliation logic).
    #[serde(default)]
    pub owned_roles: Vec<Role>,
}

impl WalletUser {
    /// Add `tabs` to the user's balance and return the new total.
    pub fn add_tabs(&mut self, tabs: i64) -> i64 {
        self.tabs += tabs;
        self.tabs
    }

    /// Deduct `cost` tabs from the user's balance and return the new total.
    ///
    /// Returns [`WalletError::NotEnoughTabs`] if the user's balance is below
    /// `cost`, leaving the balance unchanged.
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

    /// Grant the daily reward ([`DAILY_REWARD`] tabs) and return the new
    /// balance. Should only be called after confirming the user hasn't already
    /// claimed today (see [`crate::pawthos::structs::data::Data::wallet_user_daily`]).
    pub fn claim_daily(&mut self) -> i64 {
        self.add_tabs(DAILY_REWARD)
    }
}
