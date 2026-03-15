use crate::pawthos::enums::persistent_data::PersistentData;
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

    /// Wrap a DB snapshot into the correct `PersistentData` variant.
    fn to_persistent_data(db: Self::Db) -> PersistentData;
}

macro_rules! impl_user_db_spec {
    ($marker:ident, $user_type:ty, $field:ident) => {
        impl UserDbSpec for $marker {
            type Db = UserDB;
            type User = $user_type;

            fn db_lock(data: &Data) -> &RwLock<Self::Db> {
                &data.user_db
            }

            fn get_user(db: &Self::Db, user_id: UserId) -> Option<&Self::User> {
                db.get_user(user_id).map(|u| &u.$field)
            }

            fn get_user_mut(db: &mut Self::Db, user_id: UserId) -> &mut Self::User {
                &mut db.get_user_mut(user_id).$field
            }

            fn to_persistent_data(db: Self::Db) -> PersistentData {
                PersistentData::UserDB(db)
            }
        }
    };
}

impl_user_db_spec!(MimicDbMarker, MimicUser, mimic);
impl_user_db_spec!(ScheduleDbMarker, ScheduleUser, schedule);
impl_user_db_spec!(WalletDbMarker, WalletUser, wallet);
