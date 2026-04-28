//! Top-level error type for the bot.
//!
//! [`PawthosError`] is the concrete `Error` type threaded through the Poise
//! framework (see [`crate::pawthos::types::Error`]). Every domain-specific
//! error type has a `#[from]` variant here, so command handlers can use `?`
//! on any sub-system error and have it automatically wrapped.
//!
//! Two additional manual [`From`] impls handle transitive conversions that
//! `thiserror` cannot generate automatically:
//! - `From<chrono::ParseError>` → wraps via `ScheduleError::ParseError`
//! - `From<image::ImageError>` → wraps via `ColorError::ImageError`

use crate::dectalk::DectalkError;
use crate::pawthos::enums::color_errors::ColorError;
use crate::pawthos::enums::inventory_errors::InventoryError;
use crate::pawthos::enums::mimic_errors::MimicError;
use crate::pawthos::enums::profile_errors::ProfileError;
use crate::pawthos::enums::schedule_errors::ScheduleError;
use crate::pawthos::enums::wallet_errors::WalletError;

/// The single error type returned by all bot operations.
///
/// Each variant wraps one of the sub-system error types (or a third-party
/// error) and provides a human-readable `Display` message via `thiserror`.
#[derive(thiserror::Error, Debug)]
pub enum PawthosError {
    /// A Discord API or Gateway error from Serenity/Poise.
    #[error("SerenityError: {0}")]
    Serenity(#[from] poise::serenity_prelude::Error),

    /// A Tokio task panicked or was cancelled.
    #[error("tokio::JoinError: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),

    /// The persistent-data channel was closed unexpectedly.
    #[error("tokio::SendError {0}")]
    TokioSend(
        #[from]
        tokio::sync::mpsc::error::SendError<
            crate::pawthos::enums::persistent_data::PersistentData,
        >,
    ),

    /// An error from the DECtalk TTS library.
    #[error("DectalkError: {0}")]
    Dectalk(#[from] DectalkError),

    /// A standard I/O error (file read/write failures).
    #[error("std::io: {0}")]
    StdIo(#[from] std::io::Error),

    /// An error from the mimic sub-system.
    #[error("MimicError: {0}")]
    Mimic(#[from] MimicError),

    /// An error from the schedule sub-system.
    #[error("ScheduleError: {0}")]
    Schedule(#[from] ScheduleError),

    /// An error from the wallet sub-system.
    #[error("WalletError: {0}")]
    Wallet(#[from] WalletError),

    /// An error from colour parsing or image generation.
    #[error("ColorError: {0}")]
    Color(#[from] ColorError),

    /// An error from the profile sub-system.
    #[error("ProfileError: {0}")]
    Profile(#[from] ProfileError),

    /// An error from the shop / inventory sub-system.
    #[error("InventoryError: {0}")]
    Inventory(#[from] InventoryError),
}

/// Convert a `chrono::ParseError` directly into a `PawthosError` by routing
/// it through `ScheduleError::ParseError`.
impl From<chrono::ParseError> for PawthosError {
    fn from(value: chrono::ParseError) -> Self {
        PawthosError::Schedule(ScheduleError::ParseError(value))
    }
}

/// Convert an `image::ImageError` directly into a `PawthosError` by routing
/// it through `ColorError::ImageError`.
impl From<image::ImageError> for PawthosError {
    fn from(value: image::ImageError) -> Self {
        PawthosError::Color(ColorError::ImageError(value))
    }
}
