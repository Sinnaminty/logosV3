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

macro_rules! def_db_access {
    ($read_fn:ident, $write_fn:ident, $marker:ty, $user_type:ty, $err:ty, $no_user:expr) => {
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
