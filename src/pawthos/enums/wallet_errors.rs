#[derive(Debug)]
pub enum WalletError {
    NoUserFound,
    NotEnoughTabs { cost: i64, balance: i64 },
    DailyOnCooldown { remaining_secs: i64 },
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
