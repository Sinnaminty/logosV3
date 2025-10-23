use poise::serenity_prelude::{Channel, ChannelId};

use crate::{
    commands::mimic::fetch_mimics,
    types::{Context, EmbedType, PersistantData, Reply, Result},
    utils,
};

#[poise::command(slash_command, subcommands("mimic", "channel_override"))]
pub async fn delete(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// /mimic delete mimic — delete a mimic
#[poise::command(slash_command)]
pub async fn mimic(ctx: Context<'_>, #[autocomplete = "fetch_mimics"] name: String) -> Result {
    let db = &ctx.data().mimic_db;
    let mut g = db.lock().await;
    let mimic_user = g.get_user(ctx.author().id);

    let Some((index, selected_mimic)) = mimic_user
        .mimics
        .iter()
        .enumerate()
        .find(|(_, m)| m.name.eq(&name))
    else {
        let embed = utils::create_embed_builder(
            "Mimic Delete",
            "Could not find that mimic!",
            EmbedType::Bad,
        );
        ctx.send(Reply::default().embed(embed)).await?;
        //FIXME: i need to implement correct errors for these things.
        return Ok(());
    };

    let embed = utils::create_embed_builder(
        "Mimic Deleted",
        format!("You deleted \"{}\"!", selected_mimic.name),
        EmbedType::Good,
    );

    // i don't like this... but ok
    mimic_user.mimics.remove(index);

    ctx.send(Reply::default().embed(embed)).await?;

    // try a save!
    let db = g.clone();
    ctx.data()
        .persistant_data_channel
        .send(PersistantData::MimicDB(db))
        .await?;

    Ok(())
}
//FIXME: there should be a way to grab the autocomplete command data straight from
//channel_override.

/// /mimic delete channel_override — delete a channel_override if set.
#[poise::command(slash_command)]
pub async fn channel_override(ctx: Context<'_>, channel: Channel) -> Result {
    let db = &ctx.data().mimic_db;
    let mut g = db.lock().await;
    let mimic_user = g.get_user(ctx.author().id);

    let Some(_) = mimic_user
        .channel_override
        .remove(&ChannelId::from(channel.clone()))
    else {
        let embed = utils::create_embed_builder(
            "Mimic delete channel_override",
            "Could not find that channel_override!",
            EmbedType::Bad,
        );
        ctx.send(Reply::default().embed(embed)).await?;
        //FIXME: i need to implement correct errors for these things.
        return Ok(());
    };

    let embed = utils::create_embed_builder(
        "Mimic delete channel_override",
        format!(
            "Successfully deleted channel override for channel {}",
            channel
        ),
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
