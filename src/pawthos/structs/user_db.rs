use crate::pawthos::structs::{schedule_event::ScheduleEvent, user::User};
use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserDB {
    #[serde(default)]
    pub db: HashMap<UserId, User>,
}

impl UserDB {
    pub fn get_user(&self, user: UserId) -> Option<&User> {
        self.db.get(&user)
    }
    pub fn get_user_mut(&mut self, user: UserId) -> &mut User {
        self.db.entry(user).or_default()
    }

    pub fn get_events(&self) -> Vec<(UserId, ScheduleEvent)> {
        self.db
            .iter()
            .flat_map(|(id, user)| user.schedule.events.iter().map(|ev| (*id, ev.clone())))
            .collect()
    }
}
