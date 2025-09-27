use crate::{
    types::{Context, EmbedType, Error, Reply},
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
    let is_json = file
        .content_type
        .as_ref()
        .is_some_and(|ct| ct.starts_with("application/json"));

    if !is_json {
        let embed = utils::create_embed_builder(
            "OoT Add",
            "the file you provided is not a JSON file.",
            EmbedType::Bad,
        );
        ctx.send(Reply::default().embed(embed)).await?;
        return Ok(());
    }

    let embed = utils::create_embed_builder("OoT Add", "meow", EmbedType::Good);

    ctx.send(Reply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn hint(_ctx: Context<'_>) -> Result<(), Error> {
    todo!();
}
