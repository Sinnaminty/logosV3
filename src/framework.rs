use crate::commands;
use crate::handlers;
use crate::pawthos::enums::persistant_data::PersistantData;
use crate::pawthos::enums::persistant_data::UserDailyClaimed;
use crate::pawthos::enums::wallet_errors;
use crate::pawthos::structs::data::Data;
use crate::pawthos::structs::schedule_event::ScheduleEvent;
use crate::pawthos::structs::user_db::UserDB;
use crate::pawthos::types::{Error, Result};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::UserId;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::RwLock;

const BUFFER_SIZE: usize = 8;

fn save_user_db(db: UserDB) -> Result {
    let db_json = poise::serenity_prelude::json::to_string(&db)?;
    std::fs::write("user.json", db_json)?;
    log::debug!("user.json saved :3c");
    Ok(())
}
fn load_user_db() -> UserDB {
    let user_db = std::fs::read_to_string("user.json").map(serenity::json::from_str::<UserDB>);

    match user_db {
        Ok(Ok(db)) => {
            log::info!("user.json found, importing db..");
            db
        }
        Ok(Err(e)) => panic!("file is there but.. serializtion failed? {e}"), //* serializaiton failed!
        Err(_) => {
            log::warn!("user.json NOT found, making new db..");
            Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WalletList {
    date: chrono::NaiveDate,
    list: Vec<u64>,
}

fn save_wallet_list(wallet_list: WalletList) -> Result {
    const FILE_PATH: &str = "wallet_list.json";
    let wallet_list = poise::serenity_prelude::json::to_string(&wallet_list)?;
    std::fs::write(FILE_PATH, wallet_list)?;
    log::debug!("{} saved :3c", FILE_PATH);
    Ok(())
}

fn load_wallet_list() -> Result<WalletList, Error> {
    const FILE_PATH: &str = "wallet_list.json";
    let wallet_list =
        std::fs::read_to_string(FILE_PATH).map(serenity::json::from_str::<WalletList>);

    match wallet_list {
        Ok(Ok(db)) => {
            log::info!("{} found, importing..", FILE_PATH);
            Ok(db)
        }
        Ok(Err(e)) => panic!("file is there but.. serializtion failed? {e}"), //* serializaiton failed!
        Err(_) => {
            log::warn!("{} NOT found, making new..", FILE_PATH);
            Ok(Default::default())
        }
    }
}
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

pub fn setup_framework() -> poise::Framework<Data, Error> {
    let user_db = load_user_db();

    let (send, mut recv) = tokio::sync::mpsc::channel(BUFFER_SIZE);
    tokio::spawn(async move {
        while let Some(update) = recv.recv().await {
            log::debug!("update received! type: {:?}", update);
            match update {
                PersistantData::UserDB(user_db_snapshot) => {
                    if let Err(e) = save_user_db(user_db_snapshot) {
                        log::error!("Failed to save UserDB: {:?}", e);
                    }
                }
                PersistantData::DailyCheck { user_id, sender } => {
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

            let send2 = send_tasks.clone();
            user_db.get_events().into_iter().for_each(|pair| {
                send2.send(pair).unwrap();
            });

            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    user_db: RwLock::new(user_db),
                    persistant_data_channel: send,
                    schedule_events_channel: send_tasks,
                })
            })
        })
        .build()
}
