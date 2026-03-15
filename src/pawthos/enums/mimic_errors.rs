#[derive(thiserror::Error, Debug)]
pub enum MimicError {
    #[error("No Mimic User found in Mimic User Database!")]
    NoUserFound,
    #[error("User has no active Mimic Set!")]
    NoActiveMimic,
    #[error("Auto mode is false!")]
    AutoModeFalse,
    #[error("There is not a channel override set for this channel!")]
    NoChannelOverride,
    #[error("That Mimic doesn't exist!")]
    MimicNotFound,
    #[error("Cannot delete active Mimic with auto_mode enabled!")]
    DeleteActiveMimicWithAutoModeEnabled,
}
