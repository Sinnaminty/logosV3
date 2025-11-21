use crate::pawthos::{
    enums::schedule_errors::ScheduleError, structs::schedule_event::ScheduleEvent,
};
use chrono::Utc;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleUser {
    #[serde(default)]
    pub timezone: Tz,
    pub events: Vec<ScheduleEvent>,
}

impl ScheduleUser {
    /// adds an event to this ScheduleUser's events member variable.
    pub fn add_event(&mut self, name: String, when: chrono::DateTime<Utc>) -> ScheduleEvent {
        let event = ScheduleEvent {
            name,
            when,
            tz: self.timezone,
        };

        self.events.push(event.clone());
        // bruh
        self.events.sort_by_key(|e| e.when);
        event
    }

    pub fn delete_event(&mut self, target: String) -> Result<String, ScheduleError> {
        let idx = self
            .events
            .iter()
            .position(|m| m.name == target)
            .ok_or(ScheduleError::EventNotFound)?;

        let removed = self.events.remove(idx);
        Ok(removed.name)
    }

    pub fn set_timezone(&mut self, tz: Tz) {
        self.timezone = tz;
    }

    pub fn list_events(&self) -> String {
        self.events
            .iter()
            .fold(String::new(), |desc, e| desc + &e.to_string() + "\n")
    }
    pub fn prune_past_events(&mut self, now: chrono::DateTime<Utc>) {
        self.events.retain(|e| e.when > now);
    }
}
