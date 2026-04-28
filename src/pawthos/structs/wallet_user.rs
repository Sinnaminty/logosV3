//! Per-user state for the wallet/tab economy.

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::pawthos::consts::{DAILY_REWARD, MAX_STREAK_BONUS};
use crate::pawthos::enums::wallet_errors::WalletError;

/// The result of a successful `/daily` claim.
///
/// Returned by [`WalletUser::claim_daily`] so the command handler can display
/// streak progress alongside the reward.
pub struct DailyClaimResult {
    /// The user's new tab balance after the reward was added.
    pub balance: i64,
    /// How many tabs were awarded this time (base + streak bonus).
    pub reward: i64,
    /// The user's current consecutive-day streak after this claim.
    pub current_streak: u32,
}

/// All wallet-related state for a single user.
///
/// "Tabs" are the in-server currency. Users earn them via `/daily` (with a
/// streak bonus up to [`MAX_STREAK_BONUS`]) and the tab-reaction faucet, and
/// spend them in the `/shop` and on per-set cosmetics like `/profile set
/// banner` or `/shop buy rolecolor`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletUser {
    /// Current tab balance. Cannot go below zero (enforced by [`remove_tabs`]).
    ///
    /// `#[serde(default)]` means old JSON records without this field start at 0.
    ///
    /// [`remove_tabs`]: WalletUser::remove_tabs
    #[serde(default)]
    pub tabs: i64,

    /// How many consecutive days the user has claimed `/daily` without missing
    /// a day. Resets to 0 on a missed day.
    #[serde(default)]
    pub current_streak: u32,

    /// The date of the user's most recent `/daily` claim (local time).
    /// `None` if they have never claimed.
    #[serde(default)]
    pub last_claim_date: Option<NaiveDate>,
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

    /// Grant the daily reward with streak tracking.
    ///
    /// - If the user claimed yesterday, the streak increments.
    /// - Otherwise the streak resets to 1.
    /// - Bonus tabs scale with the streak up to [`MAX_STREAK_BONUS`].
    ///
    /// Should only be called after confirming the user hasn't already claimed
    /// today (see [`crate::pawthos::structs::data::Data::wallet_user_daily`]).
    pub fn claim_daily(&mut self) -> DailyClaimResult {
        let today = Local::now().date_naive();

        self.current_streak = match self.last_claim_date {
            Some(last) if last == today - chrono::Duration::days(1) => {
                self.current_streak.saturating_add(1)
            }
            _ => 1,
        };

        self.last_claim_date = Some(today);

        let bonus = (self.current_streak as i64 - 1).min(MAX_STREAK_BONUS);
        let reward = DAILY_REWARD + bonus;
        self.tabs += reward;

        DailyClaimResult {
            balance: self.tabs,
            reward,
            current_streak: self.current_streak,
        }
    }
}
