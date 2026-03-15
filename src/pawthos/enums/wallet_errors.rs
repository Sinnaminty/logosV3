//! Error type for the wallet/tab sub-system.

/// Errors that can occur when working with a user's wallet.
#[derive(Debug)]
pub enum WalletError {
    /// The calling user has no entry in the wallet database.
    ///
    /// This is a normal state for new users who have never interacted with
    /// any wallet command.
    NoUserFound,

    /// The user attempted a purchase but their tab balance is too low.
    ///
    /// `cost` is the amount required; `balance` is what the user actually has.
    /// The display message computes `difference = (balance - cost).abs()` to
    /// tell the user exactly how many tabs they are short.
    NotEnoughTabs { cost: i64, balance: i64 },

    /// The user tried to claim their daily reward but already claimed it today.
    ///
    /// `remaining_secs` is the number of seconds until the claim resets at
    /// midnight local time. The display formats this as `Xh Ym`.
    DailyOnCooldown { remaining_secs: i64 },

    /// An internal one-shot channel for the daily-check request/response
    /// cycle failed. This should never happen in normal operation.
    RecvError,
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletError::NoUserFound => write!(f, "No Wallet User found in User Database!"),
            WalletError::NotEnoughTabs { cost, balance } => {
                let difference = (balance - cost).abs();
                write!(
                    f,
                    "You don't have enough tabs to buy this! You need {difference} <:tab:1459045305084547123>."
                )
            }
            WalletError::DailyOnCooldown { remaining_secs } => {
                let hrs = remaining_secs / 3600;
                let mins = (remaining_secs % 3600) / 60;
                write!(
                    f,
                    "You already claimed your daily <:tab:1459045305084547123>. Try again in **{hrs}h {mins}m**."
                )
            }
            WalletError::RecvError => write!(f, "RecvError!! tell fizz to check logs!!!!"),
        }
    }
}

impl std::error::Error for WalletError {}
