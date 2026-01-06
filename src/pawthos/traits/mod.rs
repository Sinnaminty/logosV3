use crate::pawthos::enums::persistant_data::PersistantData;
use crate::pawthos::structs::data::Data;
use crate::pawthos::structs::mimic_user::MimicUser;
use crate::pawthos::structs::schedule_user::ScheduleUser;
use crate::pawthos::structs::user_db::UserDB;
use crate::pawthos::structs::wallet_user::WalletUser;
use poise::serenity_prelude::UserId;
use tokio::sync::RwLock;

// Marker types to distinguish DBs
pub struct MimicDbMarker;
pub struct ScheduleDbMarker;
pub struct WalletDbMarker;

/// Describes how to access and persist a particular per-user DB stored inside `Data`.
pub trait UserDbSpec {
    type Db: Clone;

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
    type Db = UserDB;
    type User = MimicUser;

    fn db_lock(data: &Data) -> &RwLock<Self::Db> {
        &data.user_db
    }

    fn get_user(db: &Self::Db, user_id: UserId) -> Option<&Self::User> {
        db.get_user(user_id).map(|u| &u.mimic)
    }

    fn get_user_mut(db: &mut Self::Db, user_id: UserId) -> &mut Self::User {
        &mut db.get_user_mut(user_id).mimic
    }

    fn to_persistant_data(db: Self::Db) -> PersistantData {
        PersistantData::UserDB(db)
    }
}

impl UserDbSpec for ScheduleDbMarker {
    type Db = UserDB;
    type User = ScheduleUser;

    fn db_lock(data: &Data) -> &RwLock<Self::Db> {
        &data.user_db
    }

    fn get_user(db: &Self::Db, user_id: UserId) -> Option<&Self::User> {
        db.get_user(user_id).map(|u| &u.schedule)
    }

    fn get_user_mut(db: &mut Self::Db, user_id: UserId) -> &mut Self::User {
        &mut db.get_user_mut(user_id).schedule
    }

    fn to_persistant_data(db: Self::Db) -> PersistantData {
        PersistantData::UserDB(db)
    }
}

impl UserDbSpec for WalletDbMarker {
    type Db = UserDB;

    type User = WalletUser;

    fn db_lock(data: &Data) -> &RwLock<Self::Db> {
        &data.user_db
    }

    fn get_user(db: &Self::Db, user_id: UserId) -> Option<&Self::User> {
        db.get_user(user_id).map(|u| &u.wallet)
    }

    fn get_user_mut(db: &mut Self::Db, user_id: UserId) -> &mut Self::User {
        &mut db.get_user_mut(user_id).wallet
    }

    fn to_persistant_data(db: Self::Db) -> PersistantData {
        PersistantData::UserDB(db)
    }
}
