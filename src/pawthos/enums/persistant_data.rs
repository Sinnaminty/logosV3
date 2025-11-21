use crate::pawthos::structs::{mimic_db::MimicDB, schedule_db::ScheduleDB};

#[derive(Debug)]
pub enum PersistantData {
    MimicDB(MimicDB),
    ScheduleDB(ScheduleDB),
}
