//! `/schedule` command suite — timezone-aware event reminders.
//!
//! Users can add named events with a date, time, and their configured timezone.
//! The bot stores events in UTC and sends the user a DM when the event time
//! arrives (via [`crate::framework`]'s reminder task).
//!
//! # Commands
//! - [`schedule`] — parent command.
//! - [`add`] — add an event (date + time + current timezone).
//! - [`list`] — list upcoming events (prunes past ones first).
//! - [`delete`] — remove an event by name.
//! - [`set_tz`] — set your home timezone (used when parsing event times).

use std::str::FromStr;

use crate::pawthos::enums::schedule_errors::ScheduleError;
use crate::pawthos::types::{Context, Result};
use crate::utils;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use chrono_tz::{TZ_VARIANTS, Tz};
use poise::serenity_prelude::{self as serenity};
use serenity::AutocompleteChoice;

// ---------------------------------------------------------------------------
// Autocomplete helper
// ---------------------------------------------------------------------------

/// Provide autocomplete choices for commands that accept an event name.
///
/// Filters the user's event list by the partial string typed so far.
async fn fetch_events(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    ctx.data()
        .with_schedule_user_read(ctx.author().id, |user| {
            Ok(user
                .events
                .iter()
                .filter_map(|e| {
                    e.name
                        .starts_with(partial)
                        .then_some(AutocompleteChoice::new(e.name.clone(), e.name.clone()))
                })
                .collect())
        })
        .await
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Schedule suite of commands for timezone-aware event reminders.
#[poise::command(slash_command, subcommands("add", "list", "delete", "set_tz"))]
pub async fn schedule(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// Add an event to your schedule and receive a DM reminder when it arrives.
///
/// Times are interpreted in your configured timezone (set with `/schedule set_tz`).
/// The event is stored in UTC internally. The bot will DM you at the event time
/// with a reminder — this persists across bot restarts.
///
/// Returns an error if the date/time string is malformed, the timezone is
/// invalid, or the time falls in a DST gap.
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Name of the event."] name: String,
    #[description = "Date (YYYY-MM-DD)"] date: String,
    #[description = "Time (HH:MM 24-hour)"] time: String,
) -> Result {
    let user_id = ctx.author().id;
    // parse date/time
    let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")?;
    let time = NaiveTime::parse_from_str(&time, "%H:%M")?;
    let naive_dt = NaiveDateTime::new(date, time);
    // now we need to grab the users timezone to create a tz datetime
    let local_tz = ctx
        .data()
        .with_schedule_user_read(user_id, |u| Ok(u.timezone))
        .await?;

    // this is the local date time :3c
    let local_dt = naive_dt
        .and_local_timezone(local_tz)
        .single()
        .ok_or(ScheduleError::AmbiguousOrInvalidTime)?;

    let embed_reply = utils::reply_ok(
        "Schedule Add",
        format!("Event {} @ {} Added to your schedule!", name, local_dt),
    );

    // no error here.. carry on
    let event = ctx
        .data()
        .with_schedule_user_write(user_id, |user| {
            let ev = user.add_event(name, local_dt.to_utc());
            Ok(ev)
        })
        .await?;
    if let Err(e) = ctx
        .data()
        .schedule_events_channel
        .send((user_id, event.clone()))
    {
        log::error!("Failed to queue reminder task! {}", e);
    }

    ctx.send(embed_reply).await?;
    Ok(())
}

/// List all of your upcoming events, sorted by time.
///
/// Past events are pruned from your list before displaying (and the pruned
/// list is saved). Each event is shown as `"<name> : <local datetime>"`.
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    let now = chrono::Utc::now();
    let reply = ctx
        .data()
        .with_schedule_user_write(user_id, |u| {
            u.prune_past_events(now);
            Ok(utils::reply_info("Schedule list", u.list_events()))
        })
        .await?;

    ctx.send(reply).await?;
    Ok(())
}

/// Delete an event from your schedule by name.
///
/// Autocomplete lists your current upcoming events.
#[poise::command(slash_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Event that you want to delete."]
    #[autocomplete = "fetch_events"]
    event: String,
) -> Result {
    let user_id = ctx.author().id;
    let event_name = ctx
        .data()
        .with_schedule_user_write(user_id, |user| user.delete_event(event))
        .await?;

    ctx.send(utils::reply_ok(
        "Schedule Delete",
        format!("Successfully deleted {} from your schedule", event_name),
    ))
    .await?;
    Ok(())
}

/// Provide autocomplete choices for timezone names.
///
/// Matches all IANA timezone strings (from `chrono_tz`) that contain the
/// partial input as a case-insensitive substring.
async fn fetch_timezones(_ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let partial = &partial.to_lowercase();

    TZ_VARIANTS
        .into_iter()
        .filter_map(|tz| {
            tz.to_string()
                .to_lowercase()
                .contains(partial)
                .then_some(AutocompleteChoice::new(tz.name(), tz.name()))
        })
        .collect()
}

/// Set your home timezone so event times are interpreted correctly.
///
/// Accepts any IANA timezone name (e.g. `America/New_York`, `Europe/London`).
/// Autocomplete searches all available timezones. Your existing events are
/// **not** adjusted — they remain stored in UTC and will display in the new
/// timezone when you next run `/schedule list`.
#[poise::command(slash_command)]
pub async fn set_tz(
    ctx: Context<'_>,
    #[description = "Timezone you want to set."]
    #[autocomplete = "fetch_timezones"]
    timezone: String,
) -> Result {
    let user_id = ctx.author().id;
    let tz = Tz::from_str(&timezone).map_err(|e| ScheduleError::InvalidTimezone(e.to_string()))?;
    ctx.data()
        .with_schedule_user_write(user_id, |user| {
            user.set_timezone(tz);
            Ok(())
        })
        .await?;

    ctx.send(utils::reply_ok(
        "Schedule set_tz",
        format!("Timezone set to {}", tz),
    ))
    .await?;

    Ok(())
}
