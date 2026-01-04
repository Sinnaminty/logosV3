use crate::commands;
use crate::handlers;
use crate::pawthos::enums::persistant_data::PersistantData;
use crate::pawthos::structs::schedule_db::ScheduleDB;
use crate::pawthos::structs::schedule_event::ScheduleEvent;
use crate::pawthos::structs::{data::Data, mimic_db::MimicDB};
use crate::pawthos::types::{Error, Result};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::User;
use poise::serenity_prelude::UserId;
use tokio::sync::RwLock;

const BUFFER_SIZE: usize = 1;

fn save_mimic_db(db: MimicDB) -> Result {
    let db_json = poise::serenity_prelude::json::to_string(&db)?;
    std::fs::write("mimic.json", db_json)?;
    log::debug!("mimic_db saved :3c");
    Ok(())
}

fn save_schedule_db(db: ScheduleDB) -> Result {
    let db_json = poise::serenity_prelude::json::to_string(&db)?;
    std::fs::write("schedule.json", db_json)?;
    log::debug!("schedule_db saved :3c");
    Ok(())
}

pub fn setup_framework() -> poise::Framework<Data, Error> {
    let mimic_db = std::fs::read_to_string("mimic.json").map(serenity::json::from_str::<MimicDB>);

    let mimic_db = match mimic_db {
        Ok(Ok(db)) => {
            log::info!("mimic.json found, importing db..");
            db
        }
        Ok(Err(e)) => panic!("file is there but.. serializtion failed? {e}"), //* serializaiton failed!
        Err(_) => {
            log::warn!("mimic.json NOT found, making new db..");
            Default::default()
        }
    };

    let schedule_db =
        std::fs::read_to_string("schedule.json").map(serenity::json::from_str::<ScheduleDB>);
    let schedule_db = match schedule_db {
        Ok(Ok(db)) => {
            log::info!("schedule.json found, importing db..");
            db
        }
        Ok(Err(e)) => panic!("file is there but.. serializtion failed? {e}"), //* serializaiton failed!
        Err(_) => {
            log::warn!("schedule.json NOT found, making new db..");
            Default::default()
        }
    };

    let (send, mut recv) = tokio::sync::mpsc::channel(BUFFER_SIZE);
    tokio::spawn(async move {
        while let Some(update) = recv.recv().await {
            log::debug!("update received! type: {:?}", update);
            match update {
                PersistantData::MimicDB(mimic_db_snapshot) => {
                    if let Err(e) = save_mimic_db(mimic_db_snapshot) {
                        log::error!("Failed to save MimicDB: {:?}", e);
                    }
                }
                PersistantData::ScheduleDB(schedule_db_snapshot) => {
                    if let Err(e) = save_schedule_db(schedule_db_snapshot) {
                        log::error!("Failed to save ScheduleDB: {:?}", e);
                    }
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
                            let time_delta = event
                                .when
                                .signed_duration_since(now)
                                .abs()
                                .to_std()
                                .expect("Time Delta should not be negative.");

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
            schedule_db.get_events().into_iter().for_each(|pair| {
                send2.send(pair).unwrap();
            });

            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    mimic_db: RwLock::new(mimic_db),
                    schedule_db: RwLock::new(schedule_db),
                    persistant_data_channel: send,
                    schedule_events_channel: send_tasks,
                })
            })
        })
        .build()
}
