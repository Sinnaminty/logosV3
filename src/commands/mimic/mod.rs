use crate::{
    commands::mimic::delete::*,
    commands::mimic::set::*,
    types::{Context, Embed, EmbedType, Mimic, PersistantData, Reply, Result},
    utils,
};

mod delete;
mod set;
use poise::serenity_prelude::{self as serenity, AutocompleteChoice, ExecuteWebhook};

async fn fetch_mimics(ctx: Context<'_>, partial: &str) -> Vec<AutocompleteChoice> {
    let all_mimics = ctx
        .data()
        .mimic_db
        .lock()
        .await
        .get_user(ctx.author().id)
        .mimics
        .clone();

    let all_mimic_names = all_mimics.into_iter().map(|m| m.name);
    let suggestions: Vec<AutocompleteChoice> = all_mimic_names
        .filter(|i| i.starts_with(partial))
        .map(|i| AutocompleteChoice::new(i.to_string(), i.to_string()))
        .collect();

    suggestions
}

#[poise::command(slash_command, subcommands("add", "list", "delete", "set", "say"))]
pub async fn mimic(_ctx: Context<'_>) -> Result {
    Ok(())
}

/// /mimic add — create a mimic from an avatar + a name.
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Name for this mimic (ignored if copying and you leave this empty)"]
    name: String,
    #[description = "Avatar URL (optional if copying from a user)"] avatar_url: Option<String>,
    #[description = "Attachment avatar (optional; overrides URL if given)"] attachment: Option<
        serenity::Attachment,
    >,
) -> Result {
    let user = ctx.author().id;

    // Decide name + avatar
    let (final_name, final_avatar) = {
        let chosen_name = name;
        let att_url = attachment.as_ref().map(|a| a.url.clone());
        let chosen_avatar = att_url.or(avatar_url);
        (chosen_name, chosen_avatar)
    };

    let mutex_db = &ctx.data().mimic_db;

    let mut g = mutex_db.lock().await;

    let mimic_user = g.get_user(user);

    let m = Mimic {
        name: final_name.clone(),
        avatar_url: final_avatar,
    };

    mimic_user.add_mimic(m.clone());
    mimic_user.active_mimic = Some(m);

    let embed = utils::create_embed_builder(
        "Mimic Add",
        format!("Success! Your mimic \"{}\" has been added :3c", final_name),
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

/// /mimic list — shows all mimics, marks the active one
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result {
    let mimic_user = ctx
        .data()
        .mimic_db
        .lock()
        .await
        .get_user(ctx.author().id)
        .clone();

    let r = mimic_user
        .mimics
        .into_iter()
        .map(|m| {
            let mut embed = Embed::new().title(m.name);
            if let Some(url) = m.avatar_url {
                embed = embed.image(url);
            }
            embed
        })
        .fold(Reply::default(), |r, e| r.embed(e));

    ctx.send(r).await?;
    Ok(())
}

/// /mimic say — speak as your active mimic in this channel
#[poise::command(slash_command)]
pub async fn say(
    ctx: Context<'_>,
    #[description = "What should your mimic say?"] text: String,
) -> Result {
    let mimic_user = ctx
        .data()
        .mimic_db
        .lock()
        .await
        .get_user(ctx.author().id)
        .clone();

    let Some(mut selected_mimic) = mimic_user.active_mimic else {
        let embed = utils::create_embed_builder(
            "Mimic Say",
            "You have no active Mimic set!",
            EmbedType::Bad,
        );

        ctx.send(Reply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    };

    //right.... time to implement logic for channel override.
    selected_mimic = match mimic_user.channel_override.get(&ctx.channel_id()) {
        Some(m) => m.clone(),
        None => selected_mimic,
    };

    let w = utils::get_or_create_webhook(ctx.http(), ctx.channel_id())
        .await
        .unwrap();

    let mut builder = ExecuteWebhook::new()
        .content(text)
        .username(selected_mimic.name);

    if let Some(s) = selected_mimic.avatar_url {
        builder = builder.avatar_url(s);
    }

    w.execute(ctx.http(), false, builder).await?;
    let a = ctx
        .send(Reply::default().ephemeral(true).content("sent~"))
        .await?;

    //delete the message :3c
    a.delete(ctx).await?;
    Ok(())
}
