//! Per-user state for the schedule feature.

use crate::pawthos::{
    enums::schedule_errors::ScheduleError, structs::schedule_event::ScheduleEvent,
};
use chrono::Utc;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

/// All schedule-related state for a single user.
///
/// Events are stored sorted by time (earliest first) so that the list
/// display is always in chronological order.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleUser {
    /// The user's home timezone, used when parsing event times and displaying
    /// the schedule list.
    ///
    /// Defaults to UTC. Set via `/schedule set_tz`.
    #[serde(default)]
    pub timezone: Tz,

    /// All upcoming events for this user, sorted ascending by [`ScheduleEvent::when`].
    pub events: Vec<ScheduleEvent>,
}

impl ScheduleUser {
    /// Add a new event and keep the list sorted by time.
    ///
    /// `when` must be in UTC. The user's current `timezone` is stored on the
    /// event so it can be displayed in local time later.
    ///
    /// Returns a clone of the newly created event (needed to enqueue the
    /// reminder task).
    pub fn add_event(&mut self, name: String, when: chrono::DateTime<Utc>) -> ScheduleEvent {
        let event = ScheduleEvent {
            name,
            when,
            tz: self.timezone,
        };

        self.events.push(event.clone());
        self.events.sort_by_key(|e| e.when);
        event
    }

    /// Remove an event by name and return its name on success.
    ///
    /// Returns [`ScheduleError::EventNotFound`] if no event with that name
    /// exists.
    pub fn delete_event(&mut self, target: String) -> Result<String, ScheduleError> {
        let idx = self
            .events
            .iter()
            .position(|m| m.name == target)
            .ok_or(ScheduleError::EventNotFound)?;

        let removed = self.events.remove(idx);
        Ok(removed.name)
    }

    /// Update the user's timezone. Does not retroactively adjust stored event
    /// times (they remain in UTC and are re-displayed in the new timezone).
    pub fn set_timezone(&mut self, tz: Tz) {
        self.timezone = tz;
    }

    /// Build a newline-separated string of all events for use in an embed
    /// description. Each line is `"<name> : <local datetime>"`.
    ///
    /// Returns an empty string if there are no events.
    pub fn list_events(&self) -> String {
        self.events
            .iter()
            .fold(String::new(), |desc, e| desc + &e.to_string() + "\n")
    }

    /// Remove all events whose `when` timestamp is in the past (before `now`).
    ///
    /// Called by `/schedule list` so stale events don't clutter the display.
    pub fn prune_past_events(&mut self, now: chrono::DateTime<Utc>) {
        self.events.retain(|e| e.when > now);
    }
}
