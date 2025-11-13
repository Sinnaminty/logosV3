use crate::pawthos::{
    enums::embed_type::EmbedType,
    structs::data::Data,
    types::{Context, Error, Reply, Result},
};
use crate::{commands::mimic::*, commands::oot::*, commands::vox::*, utils};
use poise::serenity_prelude as serenity;
pub(crate) mod mimic;
mod oot;
mod vox;

pub fn return_commands() -> Vec<poise::Command<Data, Error>> {
    vec![oot(), pfp(), register(), vox(), mimic()]
}

// NOTE: add the ability to grab both global pfp and guild.
/// Displays the calling users' profile picture
#[poise::command(slash_command)]
pub async fn pfp(
    ctx: Context<'_>,
    #[description = "User to show pfp of"] user: serenity::User,
) -> Result {
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
pub async fn register(ctx: Context<'_>) -> Result {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    log::warn!("Debug register command called!!!");
    Ok(())
}
