use crate::pawthos::enums::embed_type::EmbedType;
use crate::pawthos::structs::data::Data;
use crate::pawthos::types::Error;
use crate::pawthos::types::Reply;
use crate::utils;
use poise::FrameworkError;
use poise::serenity_prelude as serenity;
use serenity::{ExecuteWebhook, FullEvent};
use std::pin::Pin;

// TODO: rewrite error handling system in it's entirety.
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
            FullEvent::Message { new_message } => {
                // check if this user has auto mode enabled.

                let user_id = new_message.author.id;
                let channel_id = new_message.channel_id;
                let selected_mimic = data
                    .with_user_read(user_id, |maybe_user| {
                        let user = maybe_user?;
                        let auto = user.auto_mode.unwrap_or(false);
                        if !auto {
                            return None;
                        }
                        user.get_active_mimic(channel_id)
                    })
                    .await;

                let Some(selected_mimic) = selected_mimic else {
                    //BUG: this log is incorrect because having a "None" doesn't necessarliy mean
                    //that auto mode is enabled.
                    //log::warn!("auto mode enabled yet no active mimic!!");
                    return Ok(());
                };

                let content = new_message.content.clone();

                let webhook = match utils::get_or_create_webhook(&ctx.http, channel_id).await {
                    Ok(w) => w,
                    Err(e) => {
                        log::warn!("get_or_create_webhook failed: {e}");
                        return Ok(());
                    }
                };

                let mut builder = ExecuteWebhook::new()
                    .content(content)
                    .username(selected_mimic.name);

                if let Some(s) = selected_mimic.avatar_url {
                    builder = builder.avatar_url(s);
                }
                if let Err(e) = new_message.delete(&ctx.http).await {
                    log::warn!("Failed to delete original message: {e}");
                }

                if let Err(e) = webhook.execute(&ctx.http, false, builder).await {
                    log::warn!("Webhook execute failed: {e}");
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
