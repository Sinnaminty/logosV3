use crate::{commands::mimic::MimicError, pawthos::structs::mimic::Mimic};
use poise::serenity_prelude::ChannelId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MimicUser {
    pub active_mimic: Option<Mimic>,
    pub mimics: Vec<Mimic>,
    #[serde(default)]
    pub auto_mode: bool,
    pub channel_override: HashMap<ChannelId, Mimic>,
}

impl MimicUser {
    /// adds this Mimic to the mimics member variable of this user's MimicUser struct.
    pub fn add_mimic(&mut self, mimic: Mimic) {
        self.mimics.push(mimic);
    }
    /// gets this user's active_mimic, returning the correct channel_override if it exists.
    pub fn get_active_mimic(&self, channel_id: ChannelId) -> Result<Mimic, MimicError> {
        match self
            .channel_override
            .get(&channel_id)
            .cloned()
            .or_else(|| self.active_mimic.clone())
        {
            Some(m) => Ok(m),
            None => Err(MimicError::NoActiveMimic),
        }
    }
}
