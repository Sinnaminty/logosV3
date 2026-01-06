#[derive(Debug)]
pub enum WalletError {
    NoUserFound,
    NotEnoughTabs,
    DailyOnCooldown { remaining_secs: i64 },
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletError::NoUserFound => write!(f, "No Wallet User found in User Database!"),
            WalletError::NotEnoughTabs => write!(f, "You don't have enough tabs to buy this!"),
            WalletError::DailyOnCooldown { remaining_secs } => {
                write!(f, "Your daily is on cooldown!")
            }
        }
    }
}

impl std::error::Error for WalletError {}
