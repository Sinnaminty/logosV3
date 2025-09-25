use crate::{
    types::{Context, EmbedType, Error},
    utils,
};
use poise::serenity_prelude as serenity;

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
            EmbedType::Bad,
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
        EmbedType::Good,
    );
    let r = poise::reply::CreateReply::default().embed(ce);

    ctx.send(r).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn hint(_ctx: Context<'_>) -> Result<(), Error> {
    todo!();
}
