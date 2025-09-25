use crate::{
    commands::oot::*,
    types::{self, Context, Data, Error},
    utils,
};
mod oot;
use poise::serenity_prelude as serenity;

pub fn return_commands() -> Vec<poise::Command<Data, Error>> {
    vec![oot(), pfp(), register()]
}

/// Displays your or another user's account creation date
#[poise::command(slash_command)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

// NOTE: add the ability to grab both global pfp and guild.
/// Displays the calling users' profile picture
#[poise::command(slash_command)]
pub async fn pfp(
    ctx: Context<'_>,
    #[description = "User to show pfp of"] user: serenity::User,
) -> Result<(), Error> {
    let ce = utils::create_embed_builder(
        "pfp",
        format!("{}'s pfp", &user.name),
        types::EmbedType::Neutral,
    )
    .image(user.avatar_url().unwrap_or_else(|| {
        log::warn!("Slash Command Error: pfp: user.avatar_url() is None.");
        user.default_avatar_url()
    }));

    let r = poise::reply::CreateReply::default().embed(ce);
    ctx.send(r).await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    log::warn!("Debug register command called!!!");
    Ok(())
}
