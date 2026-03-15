//! Message types sent over the persistence channel.
//!
//! All database writes and daily-check requests go through a single
//! `tokio::sync::mpsc` channel to the persistence task in [`crate::framework`].
//! This keeps blocking file I/O off the async executor threads.

use crate::pawthos::structs::user_db::UserDB;

/// A message sent from a command handler to the persistence task.
///
/// The persistence task runs in a dedicated `tokio::spawn` loop and handles
/// each variant sequentially, ensuring that concurrent commands never race on
/// file I/O.
#[derive(Debug)]
pub enum PersistentData {
    /// A full snapshot of the user database to be serialised and written to
    /// `user.json`. Sent automatically after every write through
    /// [`crate::pawthos::structs::data::Data::with_db_user_write`].
    UserDB(UserDB),

    /// A request to check (and mark) whether a user has already claimed their
    /// daily reward today.
    ///
    /// The `sender` half of a one-shot channel is included so the persistence
    /// task can send the [`UserDailyClaimed`] result back to the calling
    /// command handler.
    DailyCheck {
        /// The raw user ID (as `u64`) of the user attempting to claim.
        user_id: u64,
        /// One-shot sender; the persistence task sends the claim status back
        /// through this channel.
        sender: tokio::sync::oneshot::Sender<UserDailyClaimed>,
    },
}

/// Whether a user has already claimed their daily reward for the current day.
///
/// Returned by the persistence task in response to a [`PersistentData::DailyCheck`]
/// message. The daily window resets at midnight local time.
#[derive(Debug, PartialEq, Eq)]
pub enum UserDailyClaimed {
    /// The user already claimed today — show the cooldown message.
    Claimed,
    /// The user has not yet claimed today — grant the reward.
    Unclaimed,
}
