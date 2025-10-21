use crate::{
    commands::mimic::fetch_mimics,
    types::{Context, EmbedType, PersistantData, Reply, Result},
    utils,
};
use poise::serenity_prelude::Channel;
#[poise::command(slash_command, subcommands("active_mimic", "channel_override", "auto"))]
pub async fn set(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// /mimic set active_mimic — sets your active mimic
#[poise::command(slash_command)]
pub async fn active_mimic(
    ctx: Context<'_>,
    #[autocomplete = "fetch_mimics"] name: String,
) -> Result {
    let mutex_db = &ctx.data().mimic_db;
    let mut g = mutex_db.lock().await;
    let mimic_user = g.get_user(ctx.author().id);

    let Some(selected_mimic) = mimic_user.mimics.iter().find(|m| m.name.eq(&name)) else {
        let embed = utils::create_embed_builder(
            "Mimic Set active_mimic",
            "Could not find that mimic!",
            EmbedType::Bad,
        );
        ctx.send(Reply::default().embed(embed)).await?;
        //FIXME: i need to implement correct errors for these things.
        return Ok(());
    };

    mimic_user.active_mimic = Some(selected_mimic.clone());
    let embed = utils::create_embed_builder(
        "Mimic Set",
        format!("Your active mimic is set to \"{}\"", selected_mimic.name),
        EmbedType::Good,
    );

    ctx.send(Reply::default().embed(embed)).await?;

    // try a save!
    let db = g.clone();
    ctx.data()
        .persistant_data_channel
        .send(PersistantData::MimicDB(db))
        .await?;
    Ok(())
}
/// /mimic set channel_override - overrides a channel to always display a specific mimic
#[poise::command(slash_command)]
pub async fn channel_override(
    ctx: Context<'_>,
    #[description = "What channel do you want to override?"] channel: Channel,
    #[description = "What Mimic do you want to set to this channel?"]
    #[autocomplete = "fetch_mimics"]
    name: String,
) -> Result {
    let mutex_db = &ctx.data().mimic_db;

    let mut g = mutex_db.lock().await;

    let mimic_user = g.get_user(ctx.author().id);

    let Some(selected_mimic) = mimic_user.mimics.iter().find(|m| m.name.eq(&name)) else {
        let embed = utils::create_embed_builder(
            "Mimic Set channel_override",
            "Could not find that mimic!",
            EmbedType::Bad,
        );

        ctx.send(Reply::default().embed(embed)).await?;
        //FIXME: i need to implement correct errors for these things.
        return Ok(());
    };

    mimic_user
        .channel_override
        .insert(channel.id(), selected_mimic.clone());

    let embed = utils::create_embed_builder(
        "Mimic Set channel_override",
        format!(
            "\"{}\" is set to channel \"{}\"",
            selected_mimic.name, channel
        ),
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

#[poise::command(slash_command)]
pub async fn auto(
    ctx: Context<'_>,
    #[description = "Enable/Disable Auto mode."] choice: AutoChoice,
) -> Result {
    let mutex_db = &ctx.data().mimic_db;
    let mut g = mutex_db.lock().await;
    let mimic_user = g.get_user(ctx.author().id);

    // check if this user has an active_mimic.
    if mimic_user.active_mimic.is_none() {
        let embed =
            utils::create_embed_builder("Mimic Auto", "No active mimic set!", EmbedType::Bad);
        ctx.send(Reply::default().embed(embed)).await?;
        return Ok(());
    }

    let auto_mode = mimic_user.auto_mode.insert(choice == AutoChoice::Enable);
    let embed = utils::create_embed_builder(
        "Mimic Auto",
        format!("Auto Mode: {}", auto_mode),
        EmbedType::Good,
    );
    ctx.send(Reply::default().embed(embed)).await?;

    let db = g.clone();

    ctx.data()
        .persistant_data_channel
        .send(PersistantData::MimicDB(db))
        .await?;
    Ok(())
}
