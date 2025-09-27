use crate::{
    commands::oot::*,
    types::{Context, Data, EmbedType, Error, Reply},
    utils,
};
mod oot;
use poise::serenity_prelude as serenity;

pub fn return_commands() -> Vec<poise::Command<Data, Error>> {
    vec![oot(), pfp(), register()]
}

// NOTE: add the ability to grab both global pfp and guild.
/// Displays the calling users' profile picture
#[poise::command(slash_command)]
pub async fn pfp(
    ctx: Context<'_>,
    #[description = "User to show pfp of"] user: serenity::User,
) -> Result<(), Error> {
    let ce =
        utils::create_embed_builder("pfp", format!("{}'s pfp", &user.name), EmbedType::Neutral)
            .image(user.avatar_url().unwrap_or_else(|| {
                log::warn!("Slash Command Error: pfp: user.avatar_url() is None.");
                user.default_avatar_url()
            }));

    let r = Reply::default().embed(ce);
    ctx.send(r).await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    log::warn!("Debug register command called!!!");
    Ok(())
}
