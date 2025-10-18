use crate::commands;
use crate::handlers;
use crate::types::MimicDB;
use crate::types::{Data, Error};
use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;

pub fn setup_framework() -> poise::Framework<Data, Error> {
    //TODO: add db saving.
    let mimic_db = std::fs::read_to_string("data.json").map(serenity::json::from_str::<MimicDB>);
    let db = match mimic_db {
        Ok(Ok(db)) => db,
        Ok(Err(e)) => panic!("file is there but.. serializtion failed? {e}"), //* serializaiton failed!
        Err(_) => Default::default(),
    };

    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::return_commands(),
            on_error: handlers::error_handler,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(String::from("!")),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    mimic_db: Mutex::new(db),
                })
            })
        })
        .build()
}
