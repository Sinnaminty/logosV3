use crate::{
    commands::mimic::MimicError,
    pawthos::{
        enums::embed_type::EmbedType,
        types::{Context, Reply, Result},
    },
};
use crate::{commands::mimic::fetch_mimics, utils};
use poise::serenity_prelude::Channel;

/// /mimic delete: commands meant for deleting things :3c
#[poise::command(
    slash_command,
    subcommands("mimic", "channel_override", "active_mimic")
)]
pub async fn delete(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// /mimic delete mimic: delete one of your mimics (noooo,,,,)
#[poise::command(slash_command)]
pub async fn mimic(ctx: Context<'_>, #[autocomplete = "fetch_mimics"] name: String) -> Result {
    let user_id = ctx.author().id;
    let target = name.trim();

    let deleted_mimic_name = ctx
        .data()
        .with_user_write(user_id, |user| {
            if let Some(idx) = user.mimics.iter().position(|m| m.name == target) {
                let removed = user.mimics.remove(idx);
                Ok(removed.name)
            } else {
                Err(MimicError::MimicNotFound)
            }
        })
        .await?;

    let embed = utils::create_embed_builder(
        "Mimic Delete Mimic",
        format!("You deleted \"{}\"!", deleted_mimic_name),
        EmbedType::Good,
    );

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}

/// /mimic delete channel_override: delete a channel_override if set.
#[poise::command(slash_command)]
pub async fn channel_override(ctx: Context<'_>, channel: Channel) -> Result {
    let user_id = ctx.author().id;
    let channel_id = channel.id();

    let mimic_name = ctx
        .data()
        .with_user_write(user_id, |user| {
            let Some(m) = user.channel_override.remove(&channel_id) else {
                return Err(MimicError::NoChannelOverride);
            };
            Ok(m.name)
        })
        .await?;

    let embed = utils::create_embed_builder(
        "Mimic Delete channel_override",
        format!(
            "Successfully deleted {}'s channel override for channel {}",
            mimic_name, channel
        ),
        EmbedType::Good,
    );

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}

/// /mimic delete active_mimic: deletes your active_mimic, ignoring channel_override settings.
#[poise::command(slash_command)]
pub async fn active_mimic(ctx: Context<'_>) -> Result {
    let user_id = ctx.author().id;
    let mimic_name = ctx
        .data()
        .with_user_write(user_id, |user| {
            let Some(m) = user.active_mimic.take() else {
                return Err(MimicError::NoActiveMimic);
            };

            Ok(m.name)
        })
        .await?;

    let embed = utils::create_embed_builder(
        "Mimic Delete active_mimic",
        format!("Successfully deleted your active_mimic :{}", mimic_name),
        EmbedType::Good,
    );

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}
