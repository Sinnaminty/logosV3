use crate::pawthos::structs::{schedule_event::ScheduleEvent, schedule_user::ScheduleUser};
use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, mem};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleDB {
    pub db: HashMap<UserId, ScheduleUser>,
}

impl ScheduleDB {
    /// Get an immutable reference to a ScheduleUser if they exist.
    pub fn get_user(&self, user: UserId) -> Option<&ScheduleUser> {
        self.db.get(&user)
    }

    /// Get a mutable reference to a ScheduleUser, creating one if missing.
    pub fn get_user_mut(&mut self, user: UserId) -> &mut ScheduleUser {
        self.db.entry(user).or_default()
    }
}
