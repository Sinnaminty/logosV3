//! Error type for the mimic sub-system.

/// Errors that can occur when working with a user's mimic configuration.
#[derive(thiserror::Error, Debug)]
pub enum MimicError {
    /// The calling user has no entry in the mimic database.
    ///
    /// This is a normal state for new users who have never used `/mimic add`.
    #[error("No Mimic User found in Mimic User Database!")]
    NoUserFound,

    /// The user tried to use a mimic action that requires an active mimic,
    /// but none has been set (either directly or via channel override).
    #[error("User has no active Mimic Set!")]
    NoActiveMimic,

    /// An operation was attempted that only makes sense when auto-mode is on,
    /// but the user has auto-mode disabled.
    #[error("Auto mode is false!")]
    AutoModeFalse,

    /// The user tried to delete or query a channel override for a channel
    /// that has no override configured.
    #[error("There is not a channel override set for this channel!")]
    NoChannelOverride,

    /// The requested mimic name was not found in the user's mimic list.
    #[error("That Mimic doesn't exist!")]
    MimicNotFound,

    /// The user tried to delete their active mimic while auto-mode is enabled.
    ///
    /// Auto-mode continuously uses the active mimic, so deleting it without
    /// first disabling auto-mode would leave the bot in an inconsistent state.
    #[error("Cannot delete active Mimic with auto_mode enabled!")]
    DeleteActiveMimicWithAutoModeEnabled,
}
