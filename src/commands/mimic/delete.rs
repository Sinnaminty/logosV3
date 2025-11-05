use crate::pawthos::{
    enums::embed_type::EmbedType,
    types::{Context, Reply, Result},
};
use crate::{commands::mimic::fetch_mimics, utils};
use poise::serenity_prelude::Channel;

/// /mimic delete: commands meant for deleting things :3c
#[poise::command(slash_command, subcommands("mimic", "channel_override"))]
pub async fn delete(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// /mimic delete mimic: delete one of your mimics (noooo,,,,)
#[poise::command(slash_command)]
pub async fn mimic(ctx: Context<'_>, #[autocomplete = "fetch_mimics"] name: String) -> Result {
    let user_id = ctx.author().id;
    let target = name.trim();

    let deleted: Option<String> = ctx
        .data()
        .with_user_write(user_id, |user| {
            if let Some(idx) = user.mimics.iter().position(|m| m.name == target) {
                let removed = user.mimics.remove(idx);
                Some(removed.name)
            } else {
                None
            }
        })
        .await;

    let embed = match deleted {
        Some(deleted_name) => utils::create_embed_builder(
            "Mimic Delete Mimic",
            format!("You deleted \"{}\"!", deleted_name),
            EmbedType::Good,
        ),
        None => utils::create_embed_builder(
            "Mimic Delete Mimic",
            "Could not find that mimic!",
            EmbedType::Bad,
        ),
    };

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}

/// /mimic delete channel_override: delete a channel_override if set.
#[poise::command(slash_command)]
pub async fn channel_override(ctx: Context<'_>, channel: Channel) -> Result {
    let user_id = ctx.author().id;
    let channel_id = channel.id();

    let removed: bool = ctx
        .data()
        .with_user_write(user_id, |user| {
            user.channel_override.remove(&channel_id).is_some()
        })
        .await;

    let embed = if removed {
        utils::create_embed_builder(
            "Mimic Delete channel_override",
            format!(
                "Successfully deleted channel override for channel {}",
                channel
            ),
            EmbedType::Good,
        )
    } else {
        utils::create_embed_builder(
            "Mimic Delete channel_override",
            "Could not find that channel_override!",
            EmbedType::Bad,
        )
    };

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}
