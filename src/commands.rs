use crate::{
    types::{self, Context, Data, Error},
    utils,
};

use log::warn;
use poise::{FrameworkError, serenity_prelude as serenity};
use std::pin::Pin;

pub fn return_commands() -> Vec<poise::Command<Data, Error>> {
    vec![oot(), pfp(), register()]
}

#[poise::command(slash_command, subcommands("add"), subcommand_required)]
pub async fn oot(_: Context<'_>) -> Result<(), Error> {
    //lmao
    panic!();
}

/// Add a SoH OOt Randomizer json file to Logos.
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Select your json."] file: serenity::Attachment,
) -> Result<(), Error> {
    if !(file
        .content_type
        .as_ref()
        .is_some_and(|v| v.starts_with("application/json")))
    {
        let ce = utils::create_embed_builder(
            "OoT Add",
            "The file you provided is not a json file.",
            types::EmbedType::Bad,
        );
        let r = poise::reply::CreateReply::default().embed(ce);
        ctx.send(r).await?;
        return Ok(());
    }

    let ce = utils::create_embed_builder(
        "OoT Add",
        format!(
            "File name: **{}**.\n Content type: **{}**",
            file.filename,
            file.content_type.unwrap_or(String::from("N/A"))
        ),
        types::EmbedType::Good,
    );
    let r = poise::reply::CreateReply::default().embed(ce);

    ctx.send(r).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn hint(_ctx: Context<'_>) -> Result<(), Error> {
    todo!();
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
    .image(user.avatar_url().unwrap()); //WARN: this is hot garbage. please change
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

pub fn error_handler(
    error: FrameworkError<'_, Data, Error>,
) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
    Box::pin(async move {
        match error {
            poise::FrameworkError::Command { error, ctx, .. } => {
                let _ = ctx.say(format!("Error in command: {error}")).await;
            }
            other => {
                log::error!("Framework error: {other:#?}",);
            }
        }
    })
}
