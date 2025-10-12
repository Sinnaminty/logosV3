use crate::types::{Context, Mimic, Result};
use poise::serenity_prelude as serenity;

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
    let data = ctx.data();
    let mimic_user = data.mimic_db.get_user(user);

    let m = Mimic {
        name: final_name.clone(),
        avatar_url: final_avatar,
    };

    mimic_user.add_mimic(&m);
    Ok(())
}

/// /mimic list — shows all mimics, marks the active one
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result {
    todo!();
}

/// /mimic delete — remove a mimic
#[poise::command(slash_command, guild_only)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Mimic name to delete"] name: String,
) -> Result {
    todo!();
}

/// /mimic set — sets your active mimic
#[poise::command(slash_command, guild_only)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Mimic name to activate"] name: String,
) -> Result {
    todo!();
}

/// /mimic say — speak as your active mimic in this channel

#[poise::command(slash_command, guild_only)]
pub async fn say(
    ctx: Context<'_>,
    #[description = "What should your mimic say?"] text: String,
) -> Result {
    todo!();
}
