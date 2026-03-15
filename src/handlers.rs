//! Discord event and error handlers for the Poise framework.
//!
//! Two functions are registered with the framework:
//!
//! - [`error_handler`] — called when a command or event returns an `Err`.
//!   Shows a Discord embed to the user for command errors, and auto-corrects
//!   the "auto-mode but no active mimic" edge case.
//!
//! - [`event_handler`] — called for every Discord gateway event. Only
//!   [`serenity::FullEvent::Message`] events are acted upon: if the message
//!   author has mimic auto-mode enabled, the message is re-sent via webhook
//!   as the active mimic persona and the original is deleted.

use crate::pawthos::enums::pawthos_errors::PawthosErrors;
use crate::pawthos::enums::{embed_type::EmbedType, mimic_errors::MimicError};
use crate::pawthos::structs::data::Data;
use crate::pawthos::types::Error;
use crate::pawthos::types::Reply;
use crate::utils;
use poise::FrameworkError;
use poise::serenity_prelude as serenity;
use serenity::{ExecuteWebhook, FullEvent};
use std::pin::Pin;

/// Handle errors produced by commands or event callbacks.
///
/// This function is registered as `on_error` in the framework options.
/// Poise requires a `Pin<Box<dyn Future<Output = ()> + Send + '_>>` return
/// type because it is called from a generic async context.
///
/// # Behaviour by variant
///
/// | Error variant | Action |
/// |---|---|
/// | `Command { error, ctx }` | Send a red "ERROR" embed to the invoking channel |
/// | `EventHandler { Message, NoActiveMimic }` | Disable auto-mode and notify the user |
/// | `EventHandler { Message, other }` | Log at ERROR level |
/// | Any other framework error | Log at ERROR level |
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
                // Special case: auto-mode is on but the user has no active mimic.
                // This can happen if the user deleted their active mimic without
                // first disabling auto-mode. Silently disable auto-mode and inform
                // the user so they don't wonder why their messages stopped being
                // intercepted.
                FullEvent::Message { new_message } => match error {
                    PawthosErrors::Mimic(MimicError::NoActiveMimic) => {
                        let user_id = new_message.author.id;
                        // no error to be found here..
                        let _ = framework
                            .user_data
                            .with_mimic_user_write(user_id, |user| {
                                user.auto_mode = false;
                                Ok(())
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

/// React to Discord gateway events.
///
/// Currently only handles [`FullEvent::Message`]; all other events are
/// silently ignored (debug-logged).
///
/// # Auto-mode flow
///
/// When a message arrives from a user who has `auto_mode = true`:
///
/// 1. Look up the active mimic for the channel (respecting channel overrides).
/// 2. Fetch or create the `"pawthos-mimic"` webhook for the channel.
/// 3. Execute the webhook with the mimic's name and avatar.
/// 4. Delete the original message.
///
/// If the webhook post succeeds but the delete fails, the error is returned
/// (and logged) but the webhook post is *not* undone, to avoid double-posting.
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
                    .with_mimic_user_read(user_id, |user| {
                        if !user.auto_mode {
                            return Err(MimicError::AutoModeFalse);
                        }
                        Ok(user.get_active_mimic(channel_id))
                    })
                    .await
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
                // Execute webhook first — if it fails, original message is preserved.
                if let Err(e) = webhook.execute(&ctx.http, false, builder).await {
                    log::warn!("Webhook execute failed: {e}");
                    return Err(e.into());
                }

                if let Err(e) = new_message.delete(&ctx.http).await {
                    log::warn!("Failed to delete original message: {e}");
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
