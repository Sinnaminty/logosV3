//! Short type aliases used throughout the bot.
//!
//! Centralising these aliases keeps import lists short and makes it easy
//! to swap underlying types (e.g. switching error strategy) in one place.

use crate::pawthos::enums::pawthos_errors::PawthosErrors;
use crate::pawthos::structs::data::Data;
use poise::serenity_prelude as serenity;

/// The concrete error type for all bot operations.
///
/// Every `?` in a command or event handler ultimately converts its error into
/// this type via the `#[from]` impls on [`PawthosErrors`].
pub type Error = PawthosErrors;

/// Poise command context carrying [`Data`] and [`Error`].
///
/// Provides access to `ctx.data()`, `ctx.author()`, `ctx.http()`, etc.
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// A Discord embed builder (`serenity::builder::CreateEmbed`).
pub type Embed = serenity::builder::CreateEmbed;

/// A Poise reply builder (`poise::reply::CreateReply`).
///
/// Used as the return type of the [`crate::utils::reply_ok`] family of helpers.
pub type Reply = poise::reply::CreateReply;

/// A fallible result that defaults to `()` on success and [`Error`] on failure.
///
/// Most slash command handlers return `Result` (i.e. `Result<(), Error>`).
/// Use `Result<T>` when the command needs to return a value.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;
