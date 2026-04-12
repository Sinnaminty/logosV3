//! Error type for the profile sub-system.

/// Errors that can occur when working with a user's profile.
#[derive(thiserror::Error, Debug)]
pub enum ProfileError {
    /// The calling user has no entry in the profile database.
    #[error("No Profile User found in Profile User Database!")]
    NoUserFound,

    /// The provided hex colour string could not be parsed.
    #[error("Invalid hex colour format! Use a 6-digit hex code like `FF8800` or `0xFF8800`.")]
    InvalidColorway,
}
