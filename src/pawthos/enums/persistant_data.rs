use crate::pawthos::structs::mimic_db::MimicDB;

#[derive(Debug)]
pub enum PersistantData {
    MimicDB(MimicDB),
}
