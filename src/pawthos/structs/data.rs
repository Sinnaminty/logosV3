use crate::pawthos::enums::mimic_errors::MimicError;
use crate::pawthos::enums::persistent_data::{PersistentData, UserDailyClaimed};
use crate::pawthos::enums::schedule_errors::ScheduleError;
use crate::pawthos::enums::wallet_errors::WalletError;
use crate::pawthos::structs::mimic_user::MimicUser;
use crate::pawthos::structs::schedule_event::ScheduleEvent;
use crate::pawthos::structs::schedule_user::ScheduleUser;
use crate::pawthos::structs::user_db::UserDB;
use crate::pawthos::structs::wallet_user::WalletUser;
use crate::pawthos::traits::{MimicDbMarker, ScheduleDbMarker, UserDbSpec, WalletDbMarker};
use chrono::{Duration, Local, NaiveTime};
use poise::serenity_prelude::UserId;
use tokio::sync::RwLock;

/// User data, which is stored and accessible in all command invocations
#[derive(Debug)]
pub struct Data {
    pub user_db: RwLock<UserDB>,
    pub persistent_data_channel: tokio::sync::mpsc::Sender<PersistentData>,
    pub schedule_events_channel: tokio::sync::mpsc::UnboundedSender<(UserId, ScheduleEvent)>,
}

impl Data {
    //private
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
    // Mimic
    //
    pub async fn with_mimic_user_read<R, F>(&self, user_id: UserId, f: F) -> Result<R, MimicError>
    where
        F: for<'a> FnOnce(&'a MimicUser) -> Result<R, MimicError>,
    {
        self.with_db_user_read::<MimicDbMarker, _, _>(user_id, |maybe_user| {
            let user = maybe_user.ok_or(MimicError::NoUserFound)?;
            f(user)
        })
        .await
    }
    pub async fn with_mimic_user_write<R, F>(&self, user_id: UserId, f: F) -> Result<R, MimicError>
    where
        F: for<'a> FnOnce(&'a mut MimicUser) -> Result<R, MimicError>,
    {
        self.with_db_user_write::<MimicDbMarker, _, _>(user_id, |user| f(user))
            .await
    }

    //
    // Schedule
    //
    pub async fn with_schedule_user_read<R, F>(
        &self,
        user_id: UserId,
        f: F,
    ) -> Result<R, ScheduleError>
    where
        F: for<'a> FnOnce(&'a ScheduleUser) -> Result<R, ScheduleError>,
    {
        self.with_db_user_read::<ScheduleDbMarker, _, _>(user_id, |maybe_user| {
            let user = maybe_user.ok_or(ScheduleError::NoUserFound)?;
            f(user)
        })
        .await
    }
    pub async fn with_schedule_user_write<R, F>(
        &self,
        user_id: UserId,
        f: F,
    ) -> Result<R, ScheduleError>
    where
        F: for<'a> FnOnce(&'a mut ScheduleUser) -> Result<R, ScheduleError>,
    {
        self.with_db_user_write::<ScheduleDbMarker, _, _>(user_id, |user| f(user))
            .await
    }

    //
    // Wallet
    //
    pub async fn with_wallet_user_read<R, F>(&self, user_id: UserId, f: F) -> Result<R, WalletError>
    where
        F: for<'a> FnOnce(&'a WalletUser) -> Result<R, WalletError>,
    {
        self.with_db_user_read::<WalletDbMarker, _, _>(user_id, |maybe_user| {
            let user = maybe_user.ok_or(WalletError::NoUserFound)?;
            f(user)
        })
        .await
    }
    pub async fn with_wallet_user_write<R, F>(
        &self,
        user_id: UserId,
        f: F,
    ) -> Result<R, WalletError>
    where
        F: for<'a> FnOnce(&'a mut WalletUser) -> Result<R, WalletError>,
    {
        self.with_db_user_write::<WalletDbMarker, _, _>(user_id, |user| f(user))
            .await
    }

    pub async fn wallet_user_daily(&self, user_id: UserId) -> Result<i64, WalletError> {
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
}
