use crate::pawthos::{
    enums::{embed_type::EmbedType, mimic_errors::MimicError},
    types::{Context, Reply, Result},
};
use crate::{commands::mimic::fetch_mimics, utils};
use poise::serenity_prelude::Channel;

/// /mimic set: Options to enable/disable/set for the mimic suite of commands.
#[poise::command(slash_command, subcommands("active_mimic", "channel_override", "auto"))]
pub async fn set(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// /mimic set active_mimic: Sets your active mimic
#[poise::command(slash_command)]
pub async fn active_mimic(
    ctx: Context<'_>,
    #[autocomplete = "fetch_mimics"] name: String,
) -> Result {
    let user_id = ctx.author().id;
    let target = name.trim();

    let mimic_name = ctx
        .data()
        .with_mimic_user_write(user_id, |user| {
            let m = user
                .mimics
                .iter()
                .find(|m| m.name == target)
                .ok_or(MimicError::MimicNotFound)?;
            user.active_mimic = Some(m.clone());

            Ok(m.name.clone())
        })
        .await?;

    let embed = utils::create_embed_builder(
        "Mimic Set active_mimic",
        format!("Your active mimic is set to \"{}\"", mimic_name),
        EmbedType::Good,
    );

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}
/// /mimic set channel_override: overrides a channel to always display a specific mimic
#[poise::command(slash_command)]
pub async fn channel_override(
    ctx: Context<'_>,
    #[description = "What channel do you want to override?"] channel: Channel,
    #[description = "What Mimic do you want to set to this channel?"]
    #[autocomplete = "fetch_mimics"]
    name: String,
) -> Result {
    let user_id = ctx.author().id;
    let channel_id = channel.id();
    let target = name.trim();

    let mimic_name = ctx
        .data()
        .with_mimic_user_write(user_id, |user| {
            let m = user
                .mimics
                .iter()
                .find(|m| m.name == target)
                .ok_or(MimicError::MimicNotFound)?;

            user.channel_override.insert(channel_id, m.clone());
            Ok(m.name.clone())
        })
        .await?;

    let embed = utils::create_embed_builder(
        "Mimic Set channel_override",
        format!("\"{}\" is set to channel \"{}\"", mimic_name, channel),
        EmbedType::Good,
    );

    ctx.send(Reply::default().embed(embed)).await?;
    Ok(())
}

#[derive(poise::ChoiceParameter, PartialEq)]
pub enum AutoChoice {
    #[name = "Enable"]
    Enable,
    #[name = "Disable"]
    Disable,
}

/// /mimic set auto: Automatically talk in any channel as your active mimic.
#[poise::command(slash_command)]
pub async fn auto(
    ctx: Context<'_>,
    #[description = "Enable/Disable Auto mode."] choice: AutoChoice,
) -> Result {
    let user_id = ctx.author().id;
    let enable = matches!(choice, AutoChoice::Enable);

    let outcome = ctx
        .data()
        .with_mimic_user_write(user_id, |user| {
            if user.active_mimic.is_none() {
                return Err(MimicError::NoActiveMimic);
            }
            user.auto_mode = enable;
            Ok(enable)
        })
        .await?;

    let embed = utils::create_embed_builder(
        "Mimic Auto",
        format!("Auto Mode: {}", outcome),
        EmbedType::Good,
    );

    ctx.send(Reply::default().embed(embed)).await?;

    Ok(())
}
