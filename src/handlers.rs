use crate::types::Data;
use crate::types::EmbedType;
use crate::types::Error;
use crate::types::Reply;
use crate::utils;
use poise::FrameworkError;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ExecuteWebhook;
use std::pin::Pin;

pub fn error_handler(
    error: FrameworkError<'_, Data, Error>,
) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
    Box::pin(async move {
        match error {
            poise::FrameworkError::Command { error, ctx, .. } => {
                let embed = utils::create_embed_builder(
                    "ERROR",
                    format!("Error in command: {error}"),
                    EmbedType::Bad,
                );

                let _ = ctx.send(Reply::default().embed(embed)).await;
            }
            other => {
                log::error!("Framework error: {other:#?}",);
            }
        }
    })
}

pub fn event_handler<'a>(
    ctx: &'a serenity::Context,
    event: &'a serenity::FullEvent,
    _fw_ctx: poise::FrameworkContext<'a, Data, Error>,
    data: &'a Data,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = std::result::Result<(), Error>> + std::marker::Send + 'a>,
> {
    Box::pin(async move {
        match event {
            serenity::FullEvent::Message { new_message } => {
                // check if this user has auto mode enabled.

                let mimic_user = data
                    .mimic_db
                    .lock()
                    .await
                    .get_user(new_message.author.id)
                    .clone();

                let Some(auto_mode) = mimic_user.auto_mode else {
                    return Ok(());
                };

                let Some(active_mimic) = mimic_user.active_mimic else {
                    log::warn!("Auto mode on but no active_mimic... strange..");
                    return Ok(());
                };

                if auto_mode {
                    let content = new_message.content.clone();

                    let w = utils::get_or_create_webhook(&ctx.http, new_message.channel_id)
                        .await
                        .unwrap();

                    let mut builder = ExecuteWebhook::new()
                        .content(content)
                        .username(active_mimic.name);

                    if let Some(s) = active_mimic.avatar_url {
                        builder = builder.avatar_url(s);
                    }
                    new_message.delete(&ctx.http).await?;

                    w.execute(&ctx.http, false, builder).await?;
                }
                Ok(())
            }
            _ => {
                log::debug!("event: {}", event.snake_case_name());
                Ok(())
            }
        }
    })
}
