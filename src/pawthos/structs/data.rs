use crate::pawthos::enums::persistant_data::PersistantData;
use crate::pawthos::structs::mimic_db::MimicDB;
use tokio::sync::Mutex;

/// User data, which is stored and accessible in all command invocations
#[derive(Debug)]
pub struct Data {
    pub mimic_db: Mutex<MimicDB>,
    pub persistant_data_channel: tokio::sync::mpsc::Sender<PersistantData>,
}
