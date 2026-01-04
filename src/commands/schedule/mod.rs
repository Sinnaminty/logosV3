use std::str::FromStr;

use crate::pawthos::enums::embed_type::EmbedType;
use crate::pawthos::types::{Context, Reply, Result};
use crate::utils;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use chrono_tz::{TZ_VARIANTS, Tz};
use poise::serenity_prelude::{self as serenity};
use serenity::AutocompleteChoice;

/// Returns AutocompleteChoices to the Schedule slash commands that request Event Autocompletes.
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
/// /schedule: Schedule suite of commands.
#[poise::command(slash_command, subcommands("add", "list", "delete", "set_tz"))]
pub async fn schedule(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// /schedule add: Adds an event to your schedule.
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
    // FIXME: is this unwrap dangerous?
    let local_dt = naive_dt.and_local_timezone(local_tz).unwrap();

    let embed = utils::create_embed_builder(
        "Schedule Add",
        format!("Event{} @ {} Added to your schedule!", name, local_dt),
        EmbedType::Good,
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

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}

/// /schedule list: Lists all your events.
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    let now = chrono::Utc::now();
    let embed = ctx
        .data()
        .with_schedule_user_write(user_id, |u| {
            u.prune_past_events(now);
            Ok(utils::create_embed_builder(
                "Schedule list",
                u.list_events(),
                EmbedType::Neutral,
            ))
        })
        .await?;

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}

/// /schedule delete: Deletes a selected event.
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

    let embed = utils::create_embed_builder(
        "Schedule Delete",
        format!("Successfully deleted {} from your schedule", event_name),
        EmbedType::Good,
    );

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}

async fn fetch_timezones(_ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    TZ_VARIANTS
        .into_iter()
        .filter_map(|tz| {
            tz.to_string()
                .contains(partial)
                .then_some(AutocompleteChoice::new(tz.name(), tz.name()))
        })
        .collect()
}
/// /schedule set_tz: Set the timezone you're located in.
#[poise::command(slash_command)]
pub async fn set_tz(
    ctx: Context<'_>,
    #[description = "Event that you want to delete."]
    #[autocomplete = "fetch_timezones"]
    timezone: String,
) -> Result {
    let user_id = ctx.author().id;
    let tz = Tz::from_str(&timezone).unwrap();
    ctx.data()
        .with_schedule_user_write(user_id, |user| {
            user.set_timezone(tz);
            Ok(())
        })
        .await?;

    let embed = utils::create_embed_builder(
        "Schedule set_tz",
        format!("Timezone set to {}", tz),
        EmbedType::Good,
    );

    ctx.send(Reply::default().embed(embed)).await?;

    Ok(())
}
