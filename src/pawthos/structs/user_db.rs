//! The top-level user database, persisted to `user.json`.

use crate::pawthos::structs::{schedule_event::ScheduleEvent, user::User};
use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// In-memory user database — a `HashMap` from Discord user ID to [`User`].
///
/// This is the single source of truth for all per-user state. It is held
/// behind a [`tokio::sync::RwLock`] inside [`super::data::Data`] and written
/// to disk via the persistence channel every time a user record is mutated.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserDB {
    /// The underlying map. `#[serde(default)]` means an empty JSON object
    /// (`{}`) deserialises as an empty map rather than an error.
    #[serde(default)]
    pub db: HashMap<UserId, User>,
}

impl UserDB {
    /// Return an immutable reference to a user's record, or `None` if the
    /// user has never interacted with the bot.
    pub fn get_user(&self, user: UserId) -> Option<&User> {
        self.db.get(&user)
    }

    /// Return a mutable reference to a user's record, inserting a
    /// default-constructed [`User`] if this is their first interaction.
    pub fn get_user_mut(&mut self, user: UserId) -> &mut User {
        self.db.entry(user).or_default()
    }

    /// Collect all scheduled events across every user.
    ///
    /// Called once at bot startup so the schedule reminder task can re-queue
    /// reminders for events that survived a restart.
    pub fn get_events(&self) -> Vec<(UserId, ScheduleEvent)> {
        self.db
            .iter()
            .flat_map(|(id, user)| user.schedule.events.iter().map(|ev| (*id, ev.clone())))
            .collect()
    }
}
