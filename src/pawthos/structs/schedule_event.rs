//! A single scheduled reminder event.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A named event with an absolute UTC timestamp and the user's timezone.
///
/// Events are stored in UTC so they survive the user changing their timezone,
/// but the timezone is kept alongside so reminders and the list display can
/// convert back to local time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEvent {
    /// Human-readable name of the event (chosen by the user).
    pub name: String,

    /// Absolute time of the event in UTC.
    ///
    /// The schedule reminder task sleeps until `when` and then sends the user
    /// a DM.
    pub when: chrono::DateTime<chrono::Utc>,

    /// The user's timezone at the time the event was created.
    ///
    /// Used to render the event time in the user's local time zone when
    /// listing events or sending reminders.
    pub tz: chrono_tz::Tz,
}

/// Formats the event as `"<name> : <local datetime>"`.
///
/// Used by [`super::schedule_user::ScheduleUser::list_events`] to build the
/// schedule list embed description.
impl fmt::Display for ScheduleEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} : {}", self.name, self.when.with_timezone(&self.tz))
    }
}
