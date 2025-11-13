use crate::commands::mimic::MimicError;
use crate::pawthos::enums::embed_type::EmbedType;
use crate::pawthos::enums::pawthos_errors::PawthosErrors;
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
            poise::FrameworkError::EventHandler {
                error,
                ctx,
                event,
                framework,
                ..
            } => match event {
                FullEvent::Message { new_message } => match error {
                    PawthosErrors::Mimic(MimicError::NoActiveMimic) => {
                        let user_id = new_message.author.id;
                        framework
                            .user_data
                            .with_user_write(user_id, |user| {
                                user.auto_mode = false;
                            })
                            .await;

                        if let Err(e) = new_message
                            .reply(
                                &ctx.http,
                                "You have no active mimic! unsetting auto_mode...",
                            )
                            .await
                        {
                            log::error!("super error.. {e}");
                        };
                    }
                    _ => log::error!("{error}"),
                },
                //some other FullEvent
                _ => {
                    log::error!("Framework Event error: {error}",);
                }
            },
            //some other FrameworkError
            other => {
                log::error!("Framework error: {other:#?}");
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
                let user_id = new_message.author.id;
                let channel_id = new_message.channel_id;

                // with_user_read only throws NoMimicUserFound error. if no mimic user is found in
                // the message callback, it isn't a big deal
                let selected_mimic = match data
                    .with_user_read(user_id, |user| {
                        if !user.auto_mode {
                            return Err(MimicError::AutoModeFalse);
                        }
                        Ok(user.get_active_mimic(channel_id))
                    })
                    .await
                    .flatten()
                    .flatten()
                {
                    Ok(m) => m,
                    Err(e @ (MimicError::NoUserFound | MimicError::AutoModeFalse)) => {
                        log::debug!("{e}");
                        return Ok(());
                    }
                    Err(e @ MimicError::NoActiveMimic) => {
                        log::warn!("Auto mode is true yet this user has no active mimic!");
                        return Err(e.into());
                    }
                    Err(e) => return Err(e.into()),
                };

                let content = new_message.content.clone();

                let webhook = utils::get_or_create_webhook(&ctx.http, channel_id).await?;

                let mut builder = ExecuteWebhook::new()
                    .content(content)
                    .username(selected_mimic.name);

                if let Some(s) = selected_mimic.avatar_url {
                    builder = builder.avatar_url(s);
                }
                //we also want to raise the following errors to the user.
                if let Err(e) = new_message.delete(&ctx.http).await {
                    log::warn!("Failed to delete original message: {e}");
                    return Err(e.into());
                }

                if let Err(e) = webhook.execute(&ctx.http, false, builder).await {
                    log::warn!("Webhook execute failed: {e}");
                    return Err(e.into());
                };
                Ok(())
            }
            _ => {
                log::debug!("event: {}", event.snake_case_name());
                Ok(())
            }
        }
    })
}
