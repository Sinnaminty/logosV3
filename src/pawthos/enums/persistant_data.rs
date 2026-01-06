use crate::pawthos::structs::user_db::UserDB;

#[derive(Debug)]
pub enum PersistantData {
    UserDB(UserDB),
}
