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
    let guild = ctx.guild_id().ok_or_else(|| "Cannot find guild id")?;
    let user = ctx.author().id;

    // Decide name + avatar
    let (final_name, final_avatar) = {
        let chosen_name = name;
        let att_url = attachment.as_ref().map(|a| a.url.clone());
        let chosen_avatar = att_url.or(avatar_url);
        (chosen_name, chosen_avatar)
    };

    let m = Mimic {
        name: final_name.clone(),
        avatar_url: final_avatar,
    };

    ctx.data().mimic_db;
    //put_mimic(&ctx.data().mimic_db, guild, user, &m)?;
    ctx.say(format!("✅ Added mimic **{final_name}**")).await?;
    Ok(())
}

/// /mimic list — shows all mimics, marks the active one
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result {
    let guild = ctx.guild_id().ok_or_else(|| "list")?;
    let user = ctx.author().id;
    todo!();
    //    let list = list_mimics(&ctx.data().mimic_db, guild, user)?;
    //
    //    let active = get_active(&ctx.data().mimic_db, guild, user)?;
    //
    //    if list.is_empty() {
    //        ctx.say("You have no mimics yet. Use `/mimic add`.").await?;
    //
    //        return Ok(());
    //    }
    //
    //    let mut out = String::new();
    //    for m in list {
    //        let star = active
    //            .as_deref()
    //            .map(|a| a.eq_ignore_ascii_case(&m.name))
    //            .unwrap_or(false);
    //        let mark = if star { "★" } else { "•" };
    //        let avatar = m.avatar_url.as_deref().unwrap_or("none");
    //        out.push_str(&format!("{mark} **{}** — avatar: {avatar}\n", m.name));
    //    }
    //    ctx.say(out).await?;
    //    Ok(())
}

/// /mimic delete — remove a mimic
#[poise::command(slash_command, guild_only)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Mimic name to delete"] name: String,
) -> Result {
    todo!();
    //    let guild = ctx
    //        .guild_id()
    //        .ok_or_else(|| anyhow!("Use /mimic in a server"))?;
    //    let user = ctx.author().id;
    //
    //    if del_mimic(&ctx.data().mimic_db, guild.0, user.0, &name)? {
    //        ctx.say("🗑️ Deleted.").await?;
    //    } else {
    //        ctx.say("Not found.").await?;
    //    }
    //    Ok(())
}

/// /mimic set — sets your active mimic
#[poise::command(slash_command, guild_only)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Mimic name to activate"] name: String,
) -> Result {
    todo!();
    //    let guild = ctx
    //        .guild_id()
    //        .ok_or_else(|| anyhow!("Use /mimic in a server"))?;
    //    let user = ctx.author().id;
    //
    //    set_active(&ctx.data().mimic_db, guild.0, user.0, &name)?;
    //    ctx.say(format!("⭐ Active mimic set to **{name}**"))
    //        .await?;
    //
    //    Ok(())
}

/// /mimic say — speak as your active mimic in this channel

#[poise::command(slash_command, guild_only)]
pub async fn say(
    ctx: Context<'_>,
    #[description = "What should your mimic say?"] text: String,
) -> Result {
    todo!();
    //    let guild = ctx
    //        .guild_id()
    //        .ok_or_else(|| anyhow!("Use /mimic in a server"))?;
    //    let user = ctx.author().id;
    //
    //    let Some(active_name) = get_active(&ctx.data().mimic_db, guild.0, user.0)? else {
    //        ctx.say("Set an active mimic first with `/mimic set`.")
    //            .await?;
    //        return Ok(());
    //    };
    //    let m = get_mimic(&ctx.data().mimic_db, guild.0, user.0, &active_name)?
    //        .ok_or_else(|| anyhow!("Active mimic not found (was it deleted)?"))?;
    //
    //    // Create/find webhook in this channel
    //    let hook = get_or_create_webhook(&ctx.serenity_context().http, ctx.channel_id()).await?;
    //
    //    // Send through the webhook with the mimic's name + avatar
    //    let exec = serenity::ExecuteWebhook::new()
    //        .content(text)
    //        .username(&m.name);
    //    let exec = if let Some(url) = &m.avatar_url {
    //        exec.avatar_url(url)
    //    } else {
    //        exec
    //    };
    //
    //    hook.execute(&ctx.serenity_context().http, false, exec)
    //        .await
    //        .context("Webhook execute failed (need Manage Webhooks)")?;
    //
    //    Ok(())
}
