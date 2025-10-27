use crate::commands;
use crate::handlers;
use crate::types::{Data, Error, MimicDB, PersistantData};
use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;

const BUFFER_SIZE: usize = 1;

fn save_mimic_db(db: MimicDB) -> Result<(), Error> {
    let db_json = poise::serenity_prelude::json::to_string(&db)?;
    std::fs::write("data.json", db_json)?;
    log::debug!("mimic_db saved :3c");
    Ok(())
}

pub fn setup_framework() -> poise::Framework<Data, Error> {
    let mimic_db = std::fs::read_to_string("data.json").map(serenity::json::from_str::<MimicDB>);
    let db = match mimic_db {
        Ok(Ok(db)) => {
            log::info!("data.json found, importing db..");
            db
        }
        Ok(Err(e)) => panic!("file is there but.. serializtion failed? {e}"), //* serializaiton failed!
        Err(_) => {
            log::warn!("data.json NOT found, making new db..");
            Default::default()
        }
    };

    let (send, mut recv) = tokio::sync::mpsc::channel(BUFFER_SIZE);
    tokio::spawn(async move {
        while let Some(update) = recv.recv().await {
            log::debug!("update received! type: {:?}", update);
            match update {
                PersistantData::MimicDB(mimic_db) => _ = save_mimic_db(mimic_db),
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
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    mimic_db: Mutex::new(db),
                    persistant_data_channel: send,
                })
            })
        })
        .build()
}
