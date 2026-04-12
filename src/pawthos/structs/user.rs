use crate::pawthos::structs::{mimic_user::MimicUser, schedule_user::ScheduleUser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub mimic: MimicUser,
    pub schedule: ScheduleUser,
}
