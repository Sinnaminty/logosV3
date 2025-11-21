use crate::pawthos::enums::persistant_data::PersistantData;
use crate::pawthos::structs::data::Data;
use crate::pawthos::structs::mimic_db::MimicDB;
use crate::pawthos::structs::mimic_user::MimicUser;
use crate::pawthos::structs::schedule_db::ScheduleDB;
use crate::pawthos::structs::schedule_user::ScheduleUser;
use poise::serenity_prelude::UserId;
use tokio::sync::RwLock;

// Marker types to distinguish DBs
pub struct MimicDbMarker;
pub struct ScheduleDbMarker;

/// Describes how to access and persist a particular per-user DB stored inside `Data`.
pub trait UserDbSpec {
    /// The full in-memory DB type (e.g. `MimicDB`, `ScheduleDB`).
    type Db: Clone;

    /// The per-user record type (e.g. `MimicUser`, `ScheduleUser`).
    type User;

    /// Get the `RwLock` for this DB from `Data`.
    fn db_lock(data: &Data) -> &RwLock<Self::Db>;

    /// Get an immutable view of a per-user entry.
    fn get_user(db: &Self::Db, user_id: UserId) -> Option<&Self::User>;

    /// Get a mutable per-user entry (creating a default one if needed).
    fn get_user_mut(db: &mut Self::Db, user_id: UserId) -> &mut Self::User;

    /// Wrap a DB snapshot into the correct `PersistantData` variant.
    fn to_persistant_data(db: Self::Db) -> PersistantData;
}

impl UserDbSpec for MimicDbMarker {
    type Db = MimicDB;
    type User = MimicUser;

    fn db_lock(data: &Data) -> &RwLock<Self::Db> {
        &data.mimic_db
    }

    fn get_user(db: &Self::Db, user_id: UserId) -> Option<&Self::User> {
        db.get_user(user_id)
    }

    fn get_user_mut(db: &mut Self::Db, user_id: UserId) -> &mut Self::User {
        db.get_user_mut(user_id)
    }

    fn to_persistant_data(db: Self::Db) -> PersistantData {
        PersistantData::MimicDB(db)
    }
}

impl UserDbSpec for ScheduleDbMarker {
    type Db = ScheduleDB;
    type User = ScheduleUser;

    fn db_lock(data: &Data) -> &RwLock<Self::Db> {
        &data.schedule_db
    }

    fn get_user(db: &Self::Db, user_id: UserId) -> Option<&Self::User> {
        db.get_user(user_id)
    }

    fn get_user_mut(db: &mut Self::Db, user_id: UserId) -> &mut Self::User {
        db.get_user_mut(user_id)
    }

    fn to_persistant_data(db: Self::Db) -> PersistantData {
        PersistantData::ScheduleDB(db)
    }
}
