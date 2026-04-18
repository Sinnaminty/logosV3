//! The aggregate per-user record stored in [`super::user_db::UserDB`].

use crate::pawthos::structs::{
    inventory_user::InventoryUser, mimic_user::MimicUser, profile_user::ProfileUser,
    schedule_user::ScheduleUser, wallet_user::WalletUser,
};
use serde::{Deserialize, Serialize};

/// All state associated with a single Discord user.
///
/// Each field is its own sub-struct owned by a different feature area. When a
/// new feature is added, a new field is added here and a corresponding
/// [`super::super::traits::UserDbSpec`] marker implementation routes the
/// generic DB helpers to the right field.
///
/// The `#[serde(default)]` attributes on each field ensure that old JSON
/// snapshots (which may not have all fields) deserialise cleanly into the
/// current schema.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    /// State for the `/mimic` command suite.
    #[serde(default)]
    pub mimic: MimicUser,

    /// State for the `/schedule` command suite.
    #[serde(default)]
    pub schedule: ScheduleUser,

    /// State for the wallet (`/daily`, `/balance`, `/color set`).
    #[serde(default)]
    pub wallet: WalletUser,

    /// State for the `/profile` command suite.
    #[serde(default)]
    pub profile: ProfileUser,

    /// State for the `/shop` suite — owned items, unlock flags, interaction stats.
    #[serde(default)]
    pub inventory: InventoryUser,
}
