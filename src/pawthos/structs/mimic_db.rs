use crate::pawthos::structs::mimic_user::MimicUser;
use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MimicDB {
    pub db: HashMap<UserId, MimicUser>,
}

impl MimicDB {
    /// Get an immutable reference to a user if they exist.
    pub fn get_user(&self, user: UserId) -> Option<&MimicUser> {
        self.db.get(&user)
    }

    /// Get a mutable reference to a use, creating one if missing.
    pub fn get_user_mut(&mut self, user: UserId) -> &mut MimicUser {
        self.db.entry(user).or_default()
    }
}
