//! Per-user state for the mimic feature.

use crate::{pawthos::enums::mimic_errors::MimicError, pawthos::structs::mimic::Mimic};
use poise::serenity_prelude::ChannelId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All mimic-related state for a single user.
///
/// A user can have many named mimics but only one *active* mimic at a time
/// (the one used by `/mimic say` and auto-mode). They can additionally pin
/// a specific mimic to a particular channel via `channel_override`, which
/// takes precedence over `active_mimic` in that channel.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MimicUser {
    /// The mimic that is currently "selected" as the default for this user.
    ///
    /// `None` if the user has never set an active mimic (or has deleted it).
    pub active_mimic: Option<Mimic>,

    /// All mimics this user has created, including the active one.
    pub mimics: Vec<Mimic>,

    /// When `true`, every message the user sends in any guild channel is
    /// automatically intercepted by the event handler and re-posted via
    /// webhook as the active mimic (or the channel override if one is set).
    /// The original message is deleted.
    ///
    /// Defaults to `false`. The `#[serde(default)]` attribute means old
    /// JSON records without this field deserialise as `false`.
    #[serde(default)]
    pub auto_mode: bool,

    /// Per-channel mimic overrides.
    ///
    /// When auto-mode fires in a channel that has an entry here, the override
    /// mimic is used instead of `active_mimic`. Managed via
    /// `/mimic set channel_override` and `/mimic delete channel_override`.
    pub channel_override: HashMap<ChannelId, Mimic>,
}

impl MimicUser {
    /// Append a new mimic to this user's mimic list.
    ///
    /// Does not change `active_mimic`; callers (e.g. `/mimic add`) set that
    /// separately after calling this.
    pub fn add_mimic(&mut self, mimic: Mimic) {
        self.mimics.push(mimic);
    }

    /// Return the mimic that should be used for `channel_id`.
    ///
    /// Checks `channel_override` first; falls back to `active_mimic`.
    /// Returns [`MimicError::NoActiveMimic`] if neither is set.
    pub fn get_active_mimic(&self, channel_id: ChannelId) -> Result<Mimic, MimicError> {
        self.channel_override
            .get(&channel_id)
            .cloned()
            .or_else(|| self.active_mimic.clone())
            .ok_or(MimicError::NoActiveMimic)
    }
}
