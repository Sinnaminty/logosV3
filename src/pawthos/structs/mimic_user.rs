use crate::pawthos::structs::mimic::Mimic;
use poise::serenity_prelude::ChannelId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MimicUser {
    pub active_mimic: Option<Mimic>,
    pub mimics: Vec<Mimic>,
    pub auto_mode: Option<bool>,
    #[serde(default)]
    pub channel_override: HashMap<ChannelId, Mimic>,
}

impl MimicUser {
    /// adds this Mimic to the mimics member variable of this user's MimicUser struct.
    pub fn add_mimic(&mut self, mimic: Mimic) {
        self.mimics.push(mimic);
    }
}
