#[derive(Debug)]
pub enum MimicError {
    NoUserFound,
    NoActiveMimic,
    AutoModeFalse,
    NoChannelOverride,
    MimicNotFound,
    DeleteActiveMimicWithAutoModeEnabled,
}

impl std::fmt::Display for MimicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MimicError::NoUserFound => write!(f, "No Mimic User found in Mimic User Database!"),
            MimicError::NoActiveMimic => write!(f, "User has no active Mimic Set!"),
            MimicError::AutoModeFalse => write!(f, "Auto mode is false!"),
            MimicError::NoChannelOverride => {
                write!(f, "There is not a channel override set for this channel!")
            }
            MimicError::MimicNotFound => {
                write!(f, "That Mimic doesn't exist!")
            }
            MimicError::DeleteActiveMimicWithAutoModeEnabled => {
                write!(f, "Cannot delete active Mimic with auto_mode enabled!")
            }
        }
    }
}

impl std::error::Error for MimicError {}
