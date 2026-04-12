use crate::commands::mimic::{delete::*, set::*};
use crate::pawthos::{
    enums::embed_type::EmbedType,
    structs::mimic::Mimic,
    types::{Context, Embed, Reply, Result},
};
use crate::utils::{self, create_embed_builder};
use poise::serenity_prelude as serenity;
use serenity::{AutocompleteChoice, ExecuteWebhook};
mod delete;
mod set;

/// Returns AutocompleteChoices to the Mimic slash commands that request Mimic Autocompletes.
async fn fetch_mimics(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    ctx.data()
        .with_mimic_user_read(ctx.author().id, |user| {
            Ok(user
                .mimics
                .iter()
                .filter_map(|m| {
                    m.name
                        .starts_with(partial)
                        .then_some(AutocompleteChoice::new(m.name.clone(), m.name.clone()))
                })
                .collect())
        })
        .await
        .unwrap_or_default()
}

/// /mimic: Mimic suite of commands.
#[poise::command(slash_command, subcommands("add", "list", "delete", "set", "say"))]
pub async fn mimic(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// /mimic add: Create a mimic from an avatar + a name.
// non-fallible func
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Name for this mimic"] name: String,
    #[description = "Avatar URL (optional)"] avatar_url: Option<String>,
    #[description = "Attachment avatar (optional; overrides URL if given)"] attachment: Option<
        serenity::Attachment,
    >,
) -> Result {
    let user_id = ctx.author().id;

    let att_url = attachment.as_ref().map(|a| a.url.clone());
    let avatar_url = att_url.or(avatar_url);

    // no error here...
    let _ = ctx
        .data()
        .with_mimic_user_write(user_id, |user| {
            let m = Mimic {
                name: name.clone(),
                avatar_url,
            };
            user.add_mimic(m.clone());
            user.active_mimic = Some(m);
            Ok(())
        })
        .await;

    let embed = utils::create_embed_builder(
        "Mimic Add",
        format!("Success! Your mimic \"{}\" has been added :3c", name),
        EmbedType::Good,
    );

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}

/// /mimic list: Shows a list of all mimics.
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    let reply = ctx
        .data()
        .with_mimic_user_read(user_id, |user| {
            Ok(user
                .mimics
                .iter()
                .map(|m| {
                    let mut embed = Embed::new().title(m.name.clone());
                    if let Some(url) = m.avatar_url.clone() {
                        embed = embed.image(url);
                    }
                    embed
                })
                .fold(
                    Reply::default().embed(create_embed_builder(
                        "Mimic List",
                        "",
                        EmbedType::Neutral,
                    )),
                    |r, e| r.embed(e),
                ))
        })
        .await?;

    ctx.send(reply).await?;
    Ok(())
}

/// /mimic say: Speak as your active mimic in this channel
#[poise::command(slash_command)]
pub async fn say(
    ctx: Context<'_>,
    #[description = "What should your mimic say?"] text: String,
) -> Result {
    let user_id = ctx.author().id;
    let channel_id = ctx.channel_id();
    let selected_mimic = ctx
        .data()
        .with_mimic_user_read(user_id, |user| Ok(user.get_active_mimic(channel_id)))
        .await??;

    let webhook = utils::get_or_create_webhook(ctx.http(), channel_id).await?;

    let mut builder = ExecuteWebhook::new()
        .content(text)
        .username(selected_mimic.name);

    if let Some(url) = selected_mimic.avatar_url {
        builder = builder.avatar_url(url);
    }

    webhook.execute(ctx.http(), false, builder).await?;
    let reply_handle = ctx
        .send(Reply::default().ephemeral(true).content("sent~"))
        .await?;

    //delete the message :3c
    reply_handle.delete(ctx).await?;
    Ok(())
}
