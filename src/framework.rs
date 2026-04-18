//! Poise framework construction and persistence background task.
//!
//! This module does the bulk of the bot's startup work:
//!
//! 1. **Load the user database** from `user.json` (or start fresh).
//! 2. **Spawn the persistence task** — a `tokio::spawn` loop that receives
//!    [`PersistentData`] messages and writes them to disk.  Routing all I/O
//!    through a single channel ensures that concurrent commands never race on
//!    file writes.
//! 3. **Spawn the schedule reminder task** — an outer loop receives
//!    `(UserId, ScheduleEvent)` pairs and spawns per-event `tokio::time::sleep`
//!    tasks that DM the user when the event time arrives.
//! 4. **Re-queue persisted events** — on every startup, all events currently
//!    in the database are sent to the reminder task so reminders survive bot
//!    restarts.
//! 5. **Build and return the [`poise::Framework`]**.

use crate::commands;
use crate::handlers;
use crate::pawthos::consts::FAUCET_EXPIRY_SECS;
use crate::pawthos::enums::persistent_data::PersistentData;
use crate::pawthos::enums::persistent_data::UserDailyClaimed;
use crate::pawthos::structs::data::{BountyState, Data};
use crate::pawthos::structs::schedule_event::ScheduleEvent;
use crate::pawthos::structs::user_db::UserDB;
use crate::pawthos::types::{Error, Result};
use crate::utils;
use chrono::Utc;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, MessageId, UserId};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Internal channel buffer size for the persistence task.
///
/// A small buffer is fine here because writes are cheap and the persistence
/// task keeps up easily with normal usage.
const BUFFER_SIZE: usize = 8;

// ---------------------------------------------------------------------------
// User DB persistence
// ---------------------------------------------------------------------------

/// Write `db` to `user.json` atomically (write to `.tmp`, then rename).
///
/// The atomic rename prevents a partially-written file from corrupting the
/// database if the process is killed mid-write.
fn save_user_db(db: UserDB) -> Result {
    let db_json = poise::serenity_prelude::json::to_string(&db)?;
    std::fs::write("user.json.tmp", &db_json)?;
    std::fs::rename("user.json.tmp", "user.json")?;
    log::debug!("user.json saved :3c");
    Ok(())
}

/// Load the user database from `user.json`.
///
/// Falls back to an empty [`UserDB`] if the file is absent or malformed,
/// logging a warning/error accordingly so the operator knows what happened.
fn load_user_db() -> UserDB {
    let user_db = std::fs::read_to_string("user.json").map(serenity::json::from_str::<UserDB>);

    match user_db {
        Ok(Ok(db)) => {
            log::info!("user.json found, importing db..");
            db
        }
        Ok(Err(e)) => {
            log::error!("user.json exists but deserialization failed: {e}. Starting with empty DB.");
            Default::default()
        }
        Err(_) => {
            log::warn!("user.json NOT found, making new db..");
            Default::default()
        }
    }
}

/// Run idempotent startup migrations against the in-memory [`UserDB`].
///
/// Called once right after [`load_user_db`]. Every rule checks its "is this
/// already migrated?" condition first so re-running on every startup is safe.
///
/// # Current migrations
///
/// - **Grandfather custom colorway unlock** (Phase 3): if a user already has
///   a custom colorway set but no inventory entry records the unlock, flip
///   the flag so they don't lose access to `/profile set colorway`.
/// - **Grandfather custom banner unlock** (Phase 4): analogous for banner.
fn run_migrations(user_db: &mut UserDB) {
    let mut colorway_grandfathered = 0u32;
    let mut banner_grandfathered = 0u32;

    for user in user_db.db.values_mut() {
        if user.profile.colorway.is_some() && !user.inventory.unlocked_custom_colorway {
            user.inventory.unlocked_custom_colorway = true;
            colorway_grandfathered += 1;
        }
        if user.profile.banner_url.is_some() && !user.inventory.unlocked_custom_banner {
            user.inventory.unlocked_custom_banner = true;
            banner_grandfathered += 1;
        }
    }

    if colorway_grandfathered > 0 {
        log::info!(
            "Migration: grandfathered {colorway_grandfathered} custom colorway unlock(s)",
        );
    }
    if banner_grandfathered > 0 {
        log::info!(
            "Migration: grandfathered {banner_grandfathered} custom banner unlock(s)",
        );
    }
}

// ---------------------------------------------------------------------------
// Wallet list (daily claim tracking)
// ---------------------------------------------------------------------------

/// The daily-claim tracking file, persisted as `wallet_list.json`.
///
/// The list resets automatically when [`WalletList::date`] falls behind the
/// current local date — no cron job or scheduled reset is needed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WalletList {
    /// The date for which `list` was last updated.
    date: chrono::NaiveDate,

    /// Raw user IDs of users who have already claimed today.
    list: Vec<u64>,
}

/// Write `wallet_list` to `wallet_list.json` atomically.
fn save_wallet_list(wallet_list: WalletList) -> Result {
    const FILE_PATH: &str = "wallet_list.json";
    let wallet_list_json = poise::serenity_prelude::json::to_string(&wallet_list)?;
    let tmp_path = format!("{FILE_PATH}.tmp");
    std::fs::write(&tmp_path, &wallet_list_json)?;
    std::fs::rename(&tmp_path, FILE_PATH)?;
    log::debug!("{} saved :3c", FILE_PATH);
    Ok(())
}

/// Load `wallet_list.json`, returning an empty list on missing/corrupt file.
fn load_wallet_list() -> Result<WalletList, Error> {
    const FILE_PATH: &str = "wallet_list.json";
    let wallet_list =
        std::fs::read_to_string(FILE_PATH).map(serenity::json::from_str::<WalletList>);

    match wallet_list {
        Ok(Ok(db)) => {
            log::info!("{} found, importing..", FILE_PATH);
            Ok(db)
        }
        Ok(Err(e)) => {
            log::error!("{FILE_PATH} exists but deserialization failed: {e}. Starting fresh.");
            Ok(Default::default())
        }
        Err(_) => {
            log::warn!("{} NOT found, making new..", FILE_PATH);
            Ok(Default::default())
        }
    }
}

/// Check whether user `id` has already claimed their daily reward today, and
/// if not, mark them as having claimed it.
///
/// The wallet list resets when its stored `date` is earlier than today. All
/// I/O is synchronous because this runs inside the single-threaded persistence
/// task loop (no async needed, no risk of concurrent access).
fn daily_check(id: u64) -> Result<UserDailyClaimed, Error> {
    let mut wallet_list = load_wallet_list()?;

    // in our wallet list, we want to return Ok(false) if this user did not do their daily today
    // and add them to the wallet list before saving
    // if this user did their daily today, return Ok(true)
    // or Err(e)
    let today = chrono::Local::now().date_naive();
    if wallet_list.date < today {
        wallet_list.list.clear();
        wallet_list.date = today;
    }

    let result = if wallet_list.list.contains(&id) {
        UserDailyClaimed::Claimed
    } else {
        wallet_list.list.push(id);
        UserDailyClaimed::Unclaimed
    };

    save_wallet_list(wallet_list)?;
    Ok(result)
}

// ---------------------------------------------------------------------------
// Framework construction
// ---------------------------------------------------------------------------

/// Build and return the configured [`poise::Framework`].
///
/// This is the primary entry point called from [`crate::setup`]. See the
/// module-level documentation for the full startup sequence.
pub fn setup_framework() -> poise::Framework<Data, Error> {
    let mut user_db = load_user_db();
    run_migrations(&mut user_db);

    // --- Persistence task ---------------------------------------------------
    // All DB snapshots and daily-check requests flow through this channel.
    // The task runs forever (until the process exits) and handles one message
    // at a time, serialising all file I/O.
    let (send, mut recv) = tokio::sync::mpsc::channel(BUFFER_SIZE);
    tokio::spawn(async move {
        while let Some(update) = recv.recv().await {
            log::debug!("update received! type: {:?}", update);
            match update {
                PersistentData::UserDB(user_db_snapshot) => {
                    if let Err(e) = save_user_db(user_db_snapshot) {
                        log::error!("Failed to save UserDB: {:?}", e);
                    }
                }
                PersistentData::DailyCheck { user_id, sender } => {
                    let user_daily_claimed_status = match daily_check(user_id) {
                        Ok(user_daily_claimed) => user_daily_claimed,
                        Err(e) => {
                            log::error!("Failed to save wallet_list!!: {:?}", e);

                            // if there's an error saving.. just assume that the user did not claim
                            // their daily..
                            UserDailyClaimed::Unclaimed
                        }
                    };
                    sender
                        .send(user_daily_claimed_status)
                        .expect("Receiver Channel should be open.")
                }
            };
        }
    });

    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::return_commands(),
            on_error: handlers::error_handler,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(String::from("!")),
                ..Default::default()
            },
            event_handler: handlers::event_handler,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            let http = ctx.http.clone(); // one.

            // --- Faucet state + cleanup task --------------------------------
            // Bounty state is purely in-memory: a bot restart forfeits any
            // live bounties but doesn't leak user tabs. The cleanup task
            // periodically sweeps expired bounties so the bot's reaction
            // doesn't linger on old messages.
            let faucet_bounties: Arc<RwLock<HashMap<MessageId, BountyState>>> =
                Arc::new(RwLock::new(HashMap::new()));
            let faucet_last_spawn: Arc<RwLock<Option<chrono::DateTime<Utc>>>> =
                Arc::new(RwLock::new(None));
            {
                let bounties = faucet_bounties.clone();
                let http = http.clone();
                tokio::spawn(async move {
                    let mut interval =
                        tokio::time::interval(Duration::from_secs(FAUCET_EXPIRY_SECS as u64 / 10));
                    loop {
                        interval.tick().await;
                        cleanup_expired_bounties(&bounties, &http).await;
                    }
                });
            }

            // --- Schedule reminder task -------------------------------------
            // The outer loop receives (UserId, ScheduleEvent) pairs and spawns
            // a dedicated sleep task for each one. The three clones of `http`
            // satisfy Tokio's `'static` requirement for spawned futures without
            // copying any real data (just an Arc bump).
            let (send_tasks, mut recv_tasks) =
                tokio::sync::mpsc::unbounded_channel::<(UserId, ScheduleEvent)>();
            tokio::spawn({
                let http = http.clone(); // two.
                async move {
                    while let Some((id, event)) = recv_tasks.recv().await {
                        let http = http.clone(); // three.
                        tokio::spawn(async move {
                            let now = chrono::Utc::now();
                            let Ok(time_delta) = event.when.signed_duration_since(now).to_std()
                            else {
                                log::warn!("Event in past: {:#?}", event);
                                return;
                            };

                            tokio::time::sleep(time_delta).await;
                            // send the user a message.
                            if let Ok(dm) = id.create_dm_channel(&http).await {
                                let _ = dm
                                    .say(
                                        &http,
                                        format!(
                                            "⏰ Reminder: **{}** is happening **now!**",
                                            event.name
                                        ),
                                    )
                                    .await;
                            }
                        });
                    }
                }
            });

            // Re-queue all events that survived a bot restart.
            let send2 = send_tasks.clone();
            user_db.get_events().into_iter().for_each(|pair| {
                if let Err(e) = send2.send(pair) {
                    log::error!("Failed to queue startup reminder event: {e}");
                }
            });

            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    user_db: RwLock::new(user_db),
                    persistent_data_channel: send,
                    schedule_events_channel: send_tasks,
                    faucet_bounties,
                    faucet_last_spawn,
                })
            })
        })
        .build()
}

// ---------------------------------------------------------------------------
// Faucet cleanup
// ---------------------------------------------------------------------------

/// Remove bot reactions from expired faucet bounties and drop them from the map.
///
/// Called on a timer (every `FAUCET_EXPIRY_SECS / 10` seconds, so with the
/// default 600 s expiry we run every minute). Reactions on deleted messages
/// return a Discord error which we log at debug and ignore.
async fn cleanup_expired_bounties(
    bounties: &RwLock<HashMap<MessageId, BountyState>>,
    http: &serenity::Http,
) {
    let now = Utc::now();
    let expired: Vec<(MessageId, ChannelId)> = {
        let b = bounties.read().await;
        b.iter()
            .filter(|(_, s)| s.expires_at < now)
            .map(|(id, s)| (*id, s.channel_id))
            .collect()
    };
    if expired.is_empty() {
        return;
    }

    for (msg_id, chan_id) in &expired {
        if let Err(e) = chan_id
            .delete_reaction(http, *msg_id, None, utils::tab_reaction())
            .await
        {
            log::debug!("Faucet cleanup — reaction delete on {msg_id} failed: {e}");
        }
    }

    let mut b = bounties.write().await;
    for (msg_id, _) in expired {
        b.remove(&msg_id);
    }
}
