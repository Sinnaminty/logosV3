use crate::pawthos::structs::user_db::UserDB;

#[derive(Debug)]
pub enum PersistentData {
    UserDB(UserDB),
    DailyCheck {
        user_id: u64,
        sender: tokio::sync::oneshot::Sender<UserDailyClaimed>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum UserDailyClaimed {
    Claimed,
    Unclaimed,
}
