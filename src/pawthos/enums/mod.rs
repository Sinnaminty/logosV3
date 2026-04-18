//! Domain enumerations.
//!
//! Each feature area owns its own error type so that command handlers can
//! return fine-grained errors with `?` without losing context.  All of these
//! are rolled up into the single top-level [`pawthos_errors::PawthosErrors`]
//! via `#[from]` derives, which is the concrete `Error` type threaded through
//! the Poise framework.
//!
//! | Module | Purpose |
//! |---|---|
//! | [`color_errors`] | Errors from hex-colour parsing and image generation |
//! | [`embed_type`] | Controls the accent colour of Discord embeds |
//! | [`inventory_errors`] | Errors from the shop / inventory sub-system |
//! | [`mimic_errors`] | Errors from the mimic sub-system |
//! | [`pawthos_errors`] | Top-level error enum; wraps all others |
//! | [`persistent_data`] | Messages sent over the persistence channel |
//! | [`schedule_errors`] | Errors from the schedule sub-system |
//! | [`wallet_errors`] | Errors from the wallet/tab sub-system |

pub mod color_errors;
pub mod embed_type;
pub mod inventory_errors;
pub mod mimic_errors;
pub mod pawthos_errors;
pub mod persistent_data;
pub mod profile_errors;
pub mod schedule_errors;
pub mod wallet_errors;
