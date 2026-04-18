//! The central shared-state type injected into every command invocation.
//!
//! [`Data`] is the Poise "user data" object. It lives for the lifetime of the
//! bot and is accessed through [`crate::pawthos::types::Context::data()`].
//!
//! # Database access pattern
//!
//! All reads and writes go through the [`def_db_access!`] macro-generated
//! methods (`with_*_user_read` / `with_*_user_write`). These methods:
//!
//! 1. Acquire the appropriate `RwLock` guard.
//! 2. Look up (or create) the user's record.
//! 3. Call the user-supplied closure with a reference to the sub-struct.
//! 4. On writes, clone the entire DB and send the snapshot to the persistence
//!    task via [`persistent_data_channel`] — without blocking the caller.
//!
//! [`persistent_data_channel`]: Data::persistent_data_channel

use crate::pawthos::enums::inventory_errors::InventoryError;
use crate::pawthos::enums::mimic_errors::MimicError;
use crate::pawthos::enums::persistent_data::{PersistentData, UserDailyClaimed};
use crate::pawthos::enums::profile_errors::ProfileError;
use crate::pawthos::enums::schedule_errors::ScheduleError;
use crate::pawthos::enums::wallet_errors::WalletError;
use crate::pawthos::structs::inventory_user::InventoryUser;
use crate::pawthos::structs::mimic_user::MimicUser;
use crate::pawthos::structs::profile_user::ProfileUser;
use crate::pawthos::structs::schedule_event::ScheduleEvent;
use crate::pawthos::structs::schedule_user::ScheduleUser;
use crate::pawthos::structs::shop_catalog::ACHIEVEMENTS;
use crate::pawthos::structs::user_db::UserDB;
use crate::pawthos::structs::wallet_user::{DailyClaimResult, WalletUser};
use crate::pawthos::traits::{
    InventoryDbMarker, MimicDbMarker, ProfileDbMarker, ScheduleDbMarker, UserDbSpec,
    WalletDbMarker,
};
use chrono::{DateTime, Duration, Local, NaiveTime, Utc};
use poise::serenity_prelude::{ChannelId, MessageId, UserId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Live state for a single in-flight faucet bounty.
///
/// Stored in [`Data::faucet_bounties`] keyed by the message ID the bot
/// reacted to. Removed when either (a) a user claims it by clicking the
/// reaction, or (b) the cleanup task sweeps it after
/// [`crate::pawthos::consts::FAUCET_EXPIRY_SECS`].
#[derive(Debug, Clone)]
pub struct BountyState {
    /// Channel the bountied message lives in. Needed to remove the reaction
    /// when the bounty is resolved or expires.
    pub channel_id: ChannelId,

    /// Tabs awarded to the claimer. Frozen at spawn time so changes to
    /// [`crate::pawthos::consts::FAUCET_REWARD`] don't retroactively alter
    /// existing bounties.
    pub amount: i64,

    /// UTC instant when this bounty expires. Compared against `Utc::now()`
    /// by the cleanup sweep.
    pub expires_at: DateTime<Utc>,
}

/// Shared bot state; one instance lives for the lifetime of the process.
///
/// Constructed in [`crate::framework::setup_framework`] and injected into
/// every command via the Poise framework. Cloned handles (channels) let
/// commands communicate with background tasks without holding locks.
#[derive(Debug)]
pub struct Data {
    /// The in-memory user database, protected by an async read-write lock.
    ///
    /// Multiple commands can read concurrently; writes are exclusive.
    pub user_db: RwLock<UserDB>,

    /// Sender half of the persistence channel.
    ///
    /// Every successful DB write sends a [`PersistentData::UserDB`] snapshot
    /// here so the background persistence task can flush it to disk
    /// asynchronously. Daily-check requests are also routed through this
    /// channel.
    pub persistent_data_channel: tokio::sync::mpsc::Sender<PersistentData>,

    /// Sender half of the schedule-reminder channel.
    ///
    /// Sending `(UserId, ScheduleEvent)` here causes the background scheduler
    /// to spawn a task that sleeps until the event time and then DMs the user.
    pub schedule_events_channel: tokio::sync::mpsc::UnboundedSender<(UserId, ScheduleEvent)>,

    /// Live faucet bounties, keyed by the message ID the bot reacted to.
    ///
    /// `Arc` so the cleanup task can hold its own handle. The lock is held
    /// briefly: spawn adds an entry; claim removes one; cleanup scans and
    /// removes expired entries.
    pub faucet_bounties: Arc<RwLock<HashMap<MessageId, BountyState>>>,

    /// Timestamp of the most recent faucet spawn, used to enforce
    /// [`crate::pawthos::consts::FAUCET_GLOBAL_COOLDOWN_SECS`].
    pub faucet_last_spawn: Arc<RwLock<Option<DateTime<Utc>>>>,
}

/// Generates a matching read/write method pair for one feature's user sub-struct.
///
/// # Parameters
/// - `$read_fn`  — name of the generated read method (e.g. `with_mimic_user_read`)
/// - `$write_fn` — name of the generated write method (e.g. `with_mimic_user_write`)
/// - `$marker`   — the [`UserDbSpec`] marker type (e.g. `MimicDbMarker`)
/// - `$user_type`— the concrete user sub-struct (e.g. `MimicUser`)
/// - `$err`      — the error type returned by the closure (e.g. `MimicError`)
/// - `$no_user`  — the error variant to return when the user has no DB entry
///
/// # Adding a new feature
/// Add one line inside `impl Data`:
/// ```ignore
/// def_db_access!(with_foo_user_read, with_foo_user_write, FooDbMarker, FooUser, FooError, FooError::NoUserFound);
/// ```
macro_rules! def_db_access {
    ($read_fn:ident, $write_fn:ident, $marker:ty, $user_type:ty, $err:ty, $no_user:expr) => {
        /// Read the calling user's sub-struct without modifying it.
        ///
        /// Returns `Err($no_user)` if the user has no entry in the database.
        /// The closure receives an immutable reference and must return
        /// `Result<R, $err>`.
        pub async fn $read_fn<R, F>(&self, user_id: UserId, f: F) -> Result<R, $err>
        where
            F: for<'a> FnOnce(&'a $user_type) -> Result<R, $err>,
        {
            self.with_db_user_read::<$marker, _, _>(user_id, |maybe_user| {
                let user = maybe_user.ok_or($no_user)?;
                f(user)
            })
            .await
        }

        /// Mutably access the calling user's sub-struct.
        ///
        /// Creates a default entry if the user is new. After the closure
        /// returns, the entire database is snapshotted and queued for
        /// persistence automatically — the caller does not need to do anything
        /// extra to trigger a save.
        pub async fn $write_fn<R, F>(&self, user_id: UserId, f: F) -> Result<R, $err>
        where
            F: for<'a> FnOnce(&'a mut $user_type) -> Result<R, $err>,
        {
            self.with_db_user_write::<$marker, _, _>(user_id, |user| f(user))
                .await
        }
    };
}

impl Data {
    /// Acquire a read lock and pass `Option<&User>` to a closure.
    ///
    /// Private — public callers should use the macro-generated `with_*_user_read`
    /// methods which handle the "user not found" case ergonomically.
    async fn with_db_user_read<DbMarker, R, F>(&self, user_id: UserId, f: F) -> R
    where
        DbMarker: UserDbSpec,
        F: for<'a> FnOnce(Option<&'a <DbMarker as UserDbSpec>::User>) -> R,
    {
        let lock = DbMarker::db_lock(self);
        let db_guard = lock.read().await;
        let maybe_user = DbMarker::get_user(&*db_guard, user_id);
        f(maybe_user)
    }

    /// Acquire a write lock, call the closure, then snapshot and queue the DB.
    ///
    /// Private — public callers should use the macro-generated `with_*_user_write`
    /// methods. The snapshot is sent on `persistent_data_channel`; failures are
    /// logged but not propagated to the caller.
    async fn with_db_user_write<DbMarker, R, F>(&self, user_id: UserId, f: F) -> R
    where
        DbMarker: UserDbSpec,
        F: for<'a> FnOnce(&'a mut <DbMarker as UserDbSpec>::User) -> R,
    {
        let lock = DbMarker::db_lock(self);
        let mut db_guard = lock.write().await;

        let user = DbMarker::get_user_mut(&mut *db_guard, user_id);
        let result = f(user);

        let snapshot = db_guard.clone();
        drop(db_guard);
        if let Err(e) = self
            .persistent_data_channel
            .send(DbMarker::to_persistent_data(snapshot))
            .await
        {
            log::error!("Failed to queue DB save: {:?}", e);
        }
        result
    }

    //
    // public interfaces :3c
    //

    def_db_access!(
        with_mimic_user_read,
        with_mimic_user_write,
        MimicDbMarker,
        MimicUser,
        MimicError,
        MimicError::NoUserFound
    );
    def_db_access!(
        with_schedule_user_read,
        with_schedule_user_write,
        ScheduleDbMarker,
        ScheduleUser,
        ScheduleError,
        ScheduleError::NoUserFound
    );
    def_db_access!(
        with_wallet_user_read,
        with_wallet_user_write,
        WalletDbMarker,
        WalletUser,
        WalletError,
        WalletError::NoUserFound
    );
    def_db_access!(
        with_profile_user_read,
        with_profile_user_write,
        ProfileDbMarker,
        ProfileUser,
        ProfileError,
        ProfileError::NoUserFound
    );
    def_db_access!(
        with_inventory_user_read,
        with_inventory_user_write,
        InventoryDbMarker,
        InventoryUser,
        InventoryError,
        InventoryError::NoUserFound
    );

    /// Attempt to grant the daily tab reward to a user.
    ///
    /// This method coordinates with the persistence task (via a request/response
    /// one-shot channel) to atomically check-and-mark the daily claim. The
    /// wallet list is intentionally serialised through the single-threaded
    /// persistence loop to avoid race conditions between concurrent `/daily`
    /// invocations.
    ///
    /// Returns the user's new tab balance on success, or one of:
    /// - [`WalletError::DailyOnCooldown`] — already claimed today, includes
    ///   remaining seconds until midnight.
    /// - [`WalletError::RecvError`] — the persistence channel dropped (fatal).
    pub async fn wallet_user_daily(&self, user_id: UserId) -> Result<DailyClaimResult, WalletError> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        self.persistent_data_channel
            .send(PersistentData::DailyCheck {
                user_id: user_id.into(),
                sender: tx,
            })
            .await
            .map_err(|_| WalletError::RecvError)?;

        let Ok(daily_claimed) = rx.await else {
            log::error!("recv error in DailyCheck!!");
            return Err(WalletError::RecvError);
        };

        //if daily daily_claimed, return daily error
        //else, add 10 tabs to user account and return number of tabs

        if daily_claimed == UserDailyClaimed::Claimed {
            let now = Local::now();
            let midnight = (Local::now() + Duration::days(1))
                .with_time(NaiveTime::MIN)
                .unwrap();
            let remaining = midnight - now;

            Err(WalletError::DailyOnCooldown {
                remaining_secs: remaining.num_seconds(),
            })
        } else {
            self.with_wallet_user_write(user_id, |user| Ok(user.claim_daily()))
                .await
        }
    }

    /// Return the top `limit` users sorted by tab balance (descending).
    ///
    /// Each entry is `(UserId, tabs, current_streak)`. Acquires a read lock
    /// on the full user database.
    pub async fn get_tab_leaderboard(&self, limit: usize) -> Vec<(UserId, i64, u32)> {
        let db = self.user_db.read().await;
        let mut entries: Vec<_> = db
            .db
            .iter()
            .map(|(id, user)| (*id, user.wallet.tabs, user.wallet.current_streak))
            .collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(limit);
        entries
    }

    /// Check every achievement predicate against `user_id`'s current state
    /// and unlock any that newly qualify.
    ///
    /// On each unlock, appends the achievement ID to both
    /// [`InventoryUser::unlocked_achievements`] and
    /// [`InventoryUser::owned_badges`], then posts a normal (non-ephemeral)
    /// announcement in `channel_id` so the community sees it.
    ///
    /// Safe to call after any stat mutation: the deduplication check against
    /// `unlocked_achievements` prevents re-announcing.
    ///
    /// Errors from the announcement post are logged but not returned —
    /// achievement progress is data and should never roll back because a
    /// message failed to send.
    pub async fn check_achievements(
        &self,
        user_id: UserId,
        channel_id: poise::serenity_prelude::ChannelId,
        http: &poise::serenity_prelude::Http,
    ) {
        // Snapshot both sub-structs under a single read lock.
        let snapshot = {
            let db = self.user_db.read().await;
            db.get_user(user_id)
                .map(|u| (u.inventory.clone(), u.wallet.clone()))
        };
        let Some((inv, wallet)) = snapshot else {
            return;
        };

        let newly_unlocked: Vec<&'static _> = ACHIEVEMENTS
            .iter()
            .filter(|a| (a.check)(&inv, &wallet))
            .filter(|a| !inv.unlocked_achievements.iter().any(|x| x == a.id))
            .collect();
        if newly_unlocked.is_empty() {
            return;
        }

        // Persist the unlocks.
        let _ = self
            .with_inventory_user_write(user_id, |inv| {
                for a in &newly_unlocked {
                    inv.unlocked_achievements.push(a.id.to_string());
                    inv.owned_badges.push(a.id.to_string());
                }
                Ok(())
            })
            .await;

        // Announce.
        for a in newly_unlocked {
            let content = format!(
                "🎉 <@{user_id}> unlocked an achievement: **{} {}**\n*{}*",
                a.emoji, a.name, a.description,
            );
            if let Err(e) = channel_id.say(http, content).await {
                log::warn!("Achievement announce failed: {e}");
            }
        }
    }
}
