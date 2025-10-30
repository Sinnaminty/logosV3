use crate::pawthos::{
    enums::{embed_type::EmbedType, persistant_data::PersistantData},
    types::{Context, Reply, Result},
};
use crate::{commands::mimic::fetch_mimics, utils};
use poise::serenity_prelude::{Channel, ChannelId};

/// /mimic delete: commands meant for deleting things :3c
#[poise::command(slash_command, subcommands("mimic", "channel_override"))]
pub async fn delete(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// /mimic delete mimic: delete one of your mimics (noooo,,,,)
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

/// /mimic delete channel_override: delete a channel_override if set.
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
