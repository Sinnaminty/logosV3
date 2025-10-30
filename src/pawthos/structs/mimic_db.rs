use crate::pawthos::structs::mimic_user::MimicUser;
use serde::{Deserialize, Serialize};

use poise::serenity_prelude::UserId;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MimicDB {
    pub db: HashMap<UserId, MimicUser>,
}

impl MimicDB {
    /// returns the MimicUser stored inside of the Db. will create a new MimicUser entry if the
    /// userID is not found inside of the Db.
    pub fn get_user(&mut self, user: UserId) -> &mut MimicUser {
        self.db.entry(user).or_default()
    }
}
