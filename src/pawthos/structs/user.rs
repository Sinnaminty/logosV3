use crate::pawthos::structs::{
    mimic_user::MimicUser, schedule_user::ScheduleUser, wallet_user::WalletUser,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    #[serde(default)]
    pub mimic: MimicUser,
    #[serde(default)]
    pub schedule: ScheduleUser,
    #[serde(default)]
    pub wallet: WalletUser,
}
