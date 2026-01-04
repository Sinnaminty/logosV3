use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEvent {
    pub name: String,
    pub when: chrono::DateTime<chrono::Utc>,
    pub tz: chrono_tz::Tz,
}

impl fmt::Display for ScheduleEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} : {}", self.name, self.when.with_timezone(&self.tz))
    }
}
