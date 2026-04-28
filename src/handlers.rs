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

use crate::pawthos::consts::{
    FAUCET_EXPIRY_SECS, FAUCET_GLOBAL_COOLDOWN_SECS, FAUCET_REWARD, FAUCET_TRIGGER_CHANCE,
};
use crate::pawthos::enums::pawthos_errors::PawthosErrors;
use crate::pawthos::enums::{embed_type::EmbedType, mimic_errors::MimicError};
use crate::pawthos::structs::data::{BountyState, Data};
use crate::pawthos::types::Error;
use crate::pawthos::types::Reply;
use crate::utils;
use chrono::{Duration as ChronoDuration, Utc};
use poise::FrameworkError;
use poise::serenity_prelude as serenity;
use rand::Rng;
use serenity::{ExecuteWebhook, FullEvent, Message, Reaction};
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
            FullEvent::Message { new_message } => handle_message(ctx, data, new_message).await,
            FullEvent::ReactionAdd { add_reaction } => {
                handle_reaction_add(ctx, data, add_reaction).await
            }
            _ => {
                log::debug!("event: {}", event.snake_case_name());
                Ok(())
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Message branch
// ---------------------------------------------------------------------------

/// Handle a `FullEvent::Message`.
///
/// Runs three orthogonal sub-steps on every guild message from a non-bot
/// author:
///
/// 1. Increment the user's `messages_sent` stat (drives achievements).
/// 2. Maybe spawn a faucet bounty (random + cooldown gated).
/// 3. Execute mimic auto-mode if the user has it enabled.
///
/// Bot-authored messages are ignored to avoid recursion with webhook reposts
/// from the mimic feature.
async fn handle_message(
    ctx: &serenity::Context,
    data: &Data,
    new_message: &Message,
) -> std::result::Result<(), Error> {
    if new_message.author.bot {
        return Ok(());
    }

    let user_id = new_message.author.id;
    let channel_id = new_message.channel_id;

    // Phase 7 hook: track guild-message count for achievements.
    if new_message.guild_id.is_some() {
        let _ = data
            .with_inventory_user_write(user_id, |inv| {
                inv.messages_sent = inv.messages_sent.saturating_add(1);
                Ok(())
            })
            .await;
        data.check_achievements(user_id, channel_id, &ctx.http).await;

        // Phase 5: chance to drop a faucet bounty on this message.
        try_spawn_faucet_bounty(ctx, data, new_message).await;
    }

    // --- Mimic auto-mode path (existing behaviour) -------------------------
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

// ---------------------------------------------------------------------------
// Faucet — spawn
// ---------------------------------------------------------------------------

/// Probabilistically attach a tab reaction to `new_message` and record a
/// bounty. Gated by [`FAUCET_TRIGGER_CHANCE`] and [`FAUCET_GLOBAL_COOLDOWN_SECS`].
///
/// On success, a future [`FullEvent::ReactionAdd`] by *any* user (including
/// the author) clicking the tab emoji will award them [`FAUCET_REWARD`] tabs.
async fn try_spawn_faucet_bounty(
    ctx: &serenity::Context,
    data: &Data,
    new_message: &Message,
) {
    // Cooldown check — avoid bunching bounties.
    {
        let last = data.faucet_last_spawn.read().await;
        if let Some(prev) = *last
            && (Utc::now() - prev).num_seconds() < FAUCET_GLOBAL_COOLDOWN_SECS
        {
            return;
        }
    }

    // Probabilistic trigger.
    let roll: f64 = rand::thread_rng().r#gen();
    if roll >= FAUCET_TRIGGER_CHANCE {
        return;
    }

    // Try to react first — only record the bounty if the reaction sticks.
    if let Err(e) = new_message.react(&ctx.http, utils::tab_reaction()).await {
        log::debug!("Faucet spawn — react failed: {e}");
        return;
    }

    let expires_at = Utc::now() + ChronoDuration::seconds(FAUCET_EXPIRY_SECS);
    {
        let mut bounties = data.faucet_bounties.write().await;
        bounties.insert(
            new_message.id,
            BountyState {
                channel_id: new_message.channel_id,
                amount: FAUCET_REWARD,
                expires_at,
            },
        );
    }
    *data.faucet_last_spawn.write().await = Some(Utc::now());
    log::info!(
        "Faucet spawned on message {} in channel {}",
        new_message.id,
        new_message.channel_id,
    );
}

// ---------------------------------------------------------------------------
// Faucet — claim
// ---------------------------------------------------------------------------

/// Handle a `FullEvent::ReactionAdd`.
///
/// Early-exits unless the reaction is the tab emoji AND the clicker isn't
/// the bot itself. If the message has an active bounty, awards tabs and
/// cleans up both reactions.
async fn handle_reaction_add(
    ctx: &serenity::Context,
    data: &Data,
    add_reaction: &Reaction,
) -> std::result::Result<(), Error> {
    if !utils::is_tab_reaction(&add_reaction.emoji) {
        return Ok(());
    }

    // Ignore reactions made by the bot itself (including our own spawn).
    let bot_id = ctx.cache.current_user().id;
    let Some(reactor_id) = add_reaction.user_id else {
        return Ok(());
    };
    if reactor_id == bot_id {
        return Ok(());
    }

    // Atomically pull the bounty (if any) out of the map — whoever gets here
    // first wins; subsequent clickers see `None`.
    let bounty = {
        let mut bounties = data.faucet_bounties.write().await;
        bounties.remove(&add_reaction.message_id)
    };

    let Some(bounty) = bounty else {
        return Ok(());
    };

    // Award tabs + bump the claim counter.
    data.with_wallet_user_write(reactor_id, |w| {
        w.add_tabs(bounty.amount);
        Ok(())
    })
    .await?;
    let _ = data
        .with_inventory_user_write(reactor_id, |inv| {
            inv.faucet_claims = inv.faucet_claims.saturating_add(1);
            Ok(())
        })
        .await;

    // Remove both reactions. Removing the claimer's may fail without
    // MANAGE_MESSAGES — log & swallow so the payout still happens.
    if let Err(e) = bounty
        .channel_id
        .delete_reaction(&ctx.http, add_reaction.message_id, None, utils::tab_reaction())
        .await
    {
        log::debug!("Faucet claim — failed to remove bot reaction: {e}");
    }
    if let Err(e) = bounty
        .channel_id
        .delete_reaction(
            &ctx.http,
            add_reaction.message_id,
            Some(reactor_id),
            utils::tab_reaction(),
        )
        .await
    {
        log::debug!("Faucet claim — failed to remove claimer reaction: {e}");
    }

    log::info!(
        "Faucet claimed: user {} got {} tabs on message {}",
        reactor_id,
        bounty.amount,
        add_reaction.message_id,
    );

    // Achievement check — "Quick Fingers" unlocks here.
    data.check_achievements(reactor_id, bounty.channel_id, &ctx.http)
        .await;
    Ok(())
}
