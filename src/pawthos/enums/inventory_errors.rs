//! Error type for the shop / inventory sub-system.

use crate::pawthos::enums::wallet_errors::WalletError;

/// Errors that can occur when working with a user's inventory or the shop.
#[derive(thiserror::Error, Debug)]
pub enum InventoryError {
    /// The calling user has no entry in the inventory database.
    #[error("No Inventory User found in User Database!")]
    NoUserFound,

    /// No catalog entry matches the given item ID.
    #[error("Unknown item id: `{0}`.")]
    UnknownItem(String),

    /// The user already owns this item.
    #[error("You already own **{0}**.")]
    AlreadyOwned(String),

    /// The user does not own this item (required for equipping, gifting, etc.).
    #[error("You don't own **{0}**.")]
    NotOwned(String),

    /// A gift command was invoked with the sender as the recipient.
    #[error("You can't gift items to yourself.")]
    GiftToSelf,

    /// The recipient of a gift already owns the item.
    #[error("{0} already owns **{1}**.")]
    RecipientOwns(String, String),

    /// The user tried to use a custom-* feature without the unlock purchased.
    #[error("Custom {0} is locked — purchase the unlock in `/shop` first.")]
    FeatureLocked(&'static str),

    /// The user attempted to equip more badges than [`crate::pawthos::consts::MAX_ACTIVE_BADGES`].
    #[error("Can't equip more than {max} badges (you tried {attempted}).")]
    TooManyBadges { max: usize, attempted: usize },

    /// A user-supplied custom string exceeded its length limit.
    #[error("Custom {field} exceeds {max}-char limit.")]
    CustomTooLong {
        field: &'static str,
        max: usize,
    },

    /// Reaction was added to a message that has no active faucet bounty.
    #[error("No faucet bounty is active on that message.")]
    NoFaucetBounty,

    /// Wrap wallet errors so purchase flows can use `?` uniformly.
    #[error(transparent)]
    Wallet(#[from] WalletError),
}
