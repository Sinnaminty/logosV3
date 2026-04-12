//! Marker-trait system for generic, type-safe database access.
//!
//! # Design
//!
//! Each feature area (mimic, schedule, wallet) has its own sub-struct inside
//! [`super::structs::user::User`]. To avoid writing nearly-identical read/write
//! methods for each one, the [`UserDbSpec`] trait abstracts over "which
//! sub-struct does this marker type refer to?".
//!
//! The [`impl_user_db_spec!`] macro then generates the boilerplate `impl`
//! blocks from a single line each.
//!
//! # Adding a new feature
//!
//! 1. Add `pub struct NewFeatureDbMarker;` below the existing markers.
//! 2. Add one `impl_user_db_spec!(NewFeatureDbMarker, NewFeatureUser, new_field);` call.
//! 3. Add one `def_db_access!(...)` call in [`super::structs::data`].

use crate::pawthos::enums::persistent_data::PersistentData;
use crate::pawthos::structs::data::Data;
use crate::pawthos::structs::mimic_user::MimicUser;
use crate::pawthos::structs::profile_user::ProfileUser;
use crate::pawthos::structs::schedule_user::ScheduleUser;
use crate::pawthos::structs::user_db::UserDB;
use crate::pawthos::structs::wallet_user::WalletUser;
use poise::serenity_prelude::UserId;
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// Marker types — one per database sub-system
// ---------------------------------------------------------------------------

/// Marker type that routes generic DB operations to the mimic sub-struct.
pub struct MimicDbMarker;

/// Marker type that routes generic DB operations to the schedule sub-struct.
pub struct ScheduleDbMarker;

/// Marker type that routes generic DB operations to the wallet sub-struct.
pub struct WalletDbMarker;

/// Marker type that routes generic DB operations to the profile sub-struct.
pub struct ProfileDbMarker;

// ---------------------------------------------------------------------------
// Trait definition
// ---------------------------------------------------------------------------

/// Describes how to access and persist a particular per-user sub-struct
/// stored inside [`Data`].
///
/// Implement this trait (via [`impl_user_db_spec!`]) for each marker type to
/// teach the generic helpers in [`super::structs::data::Data`] where to find
/// the right data.
pub trait UserDbSpec {
    /// The top-level database type (always [`UserDB`] for now).
    type Db: Clone;

    /// The per-user sub-struct this marker routes to (e.g. `MimicUser`).
    type User;

    /// Return the `RwLock` that guards this database inside `data`.
    fn db_lock(data: &Data) -> &RwLock<Self::Db>;

    /// Look up an immutable reference to the user's sub-struct.
    ///
    /// Returns `None` if the user has never interacted with the bot.
    fn get_user(db: &Self::Db, user_id: UserId) -> Option<&Self::User>;

    /// Look up a mutable reference to the user's sub-struct, inserting a
    /// default entry if the user is new.
    fn get_user_mut(db: &mut Self::Db, user_id: UserId) -> &mut Self::User;

    /// Wrap a database snapshot into the correct [`PersistentData`] variant
    /// so the persistence task knows which file to write.
    fn to_persistent_data(db: Self::Db) -> PersistentData;
}

// ---------------------------------------------------------------------------
// Macro
// ---------------------------------------------------------------------------

/// Generate a [`UserDbSpec`] implementation for a marker type.
///
/// # Usage
/// ```ignore
/// impl_user_db_spec!(MimicDbMarker, MimicUser, mimic);
/// // expands to: impl UserDbSpec for MimicDbMarker { ... }
/// ```
///
/// # Parameters
/// - `$marker`    — the marker struct (e.g. `MimicDbMarker`)
/// - `$user_type` — the concrete user sub-struct (e.g. `MimicUser`)
/// - `$field`     — the field name on [`super::structs::user::User`] (e.g. `mimic`)
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

// ---------------------------------------------------------------------------
// Implementations
// ---------------------------------------------------------------------------

impl_user_db_spec!(MimicDbMarker, MimicUser, mimic);
impl_user_db_spec!(ScheduleDbMarker, ScheduleUser, schedule);
impl_user_db_spec!(WalletDbMarker, WalletUser, wallet);
impl_user_db_spec!(ProfileDbMarker, ProfileUser, profile);
