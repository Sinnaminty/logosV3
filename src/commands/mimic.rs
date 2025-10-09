use anyhow::{Context, Result, anyhow};
use poise::serenity_prelude as serenity;

use serde::{Deserialize, Serialize};
use sled::Db;

use std::{path::Path, sync::Arc};

type UserId = u64;

type GuildId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mimic {
    pub name: String,
    pub avatar_url: Option<String>,
}

impl Mimic {
    fn key(guild: GuildId, user: UserId, name: &str) -> String {
        format!("mimic/{guild}/{user}/{}", name.to_lowercase())
    }
}

fn active_key(guild: GuildId, user: UserId) -> String {
    format!("active/{guild}/{user}")
}

pub struct AppData {
    pub db: Arc<Db>,
}

type Ctx<'a> = poise::Context<'a, AppData, anyhow::Error>;

async fn get_or_create_webhook(
    http: &serenity::Http,
    channel_id: serenity::ChannelId,
) -> Result<serenity::Webhook> {
    const WEBHOOK_NAME: &str = "logosV3-mimic";
    if let Ok(existing) = channel_id.webhooks(http).await {
        if let Some(w) = existing
            .into_iter()
            .find(|w| w.name.as_deref() == Some(WEBHOOK_NAME))
        {
            return Ok(w);
        }
    }
    let hook = channel_id
        .create_webhook(http, serenity::CreateWebhook::new(WEBHOOK_NAME))
        .await
        .context("Failed to create webhook (need Manage Webhooks permission)")?;
    Ok(hook)
}

fn put_mimic(db: &Db, guild: GuildId, user: UserId, m: &Mimic) -> Result<()> {
    let key = Mimic::key(guild, user, &m.name);
    let val = serde_json::to_vec(m)?;
    db.insert(key.as_bytes(), val)?;
    db.flush()?;
    Ok(())
}

fn del_mimic(db: &Db, guild: GuildId, user: UserId, name: &str) -> Result<bool> {
    let key = Mimic::key(guild, user, name);
    let existed = db.remove(key.as_bytes())?.is_some();
    if existed {
        // if it was active, clear it
        let ak = active_key(guild, user);
        if let Some(cur) = db.get(ak.as_bytes())? {
            let cur_name: String = serde_json::from_slice(&cur)?;
            if cur_name.eq_ignore_ascii_case(name) {
                let _ = db.remove(ak.as_bytes())?;
            }
        }
        db.flush()?;
    }
    Ok(existed)
}

fn list_mimics(db: &Db, guild: GuildId, user: UserId) -> Result<Vec<Mimic>> {
    let prefix = format!("mimic/{guild}/{user}/");
    let mut v = Vec::new();
    for item in db.scan_prefix(prefix.as_bytes()) {
        let (_, bytes) = item?;
        v.push(serde_json::from_slice::<Mimic>(&bytes)?);
    }
    Ok(v)
}

fn set_active(db: &Db, guild: GuildId, user: UserId, name: &str) -> Result<()> {
    // ensure exists
    let key = Mimic::key(guild, user, name);
    if db.get(key.as_bytes())?.is_none() {
        return Err(anyhow!("No mimic named `{name}` found"));
    }
    let ak = active_key(guild, user);
    db.insert(ak.as_bytes(), serde_json::to_vec(&name)?)?;
    db.flush()?;
    Ok(())
}

fn get_active(db: &Db, guild: GuildId, user: UserId) -> Result<Option<String>> {
    let ak = active_key(guild, user);
    Ok(db
        .get(ak.as_bytes())?
        .map(|v| serde_json::from_slice::<String>(&v))
        .transpose()?)
}

fn get_mimic(db: &Db, guild: GuildId, user: UserId, name: &str) -> Result<Option<Mimic>> {
    let key = Mimic::key(guild, user, name);
    Ok(db
        .get(key.as_bytes())?
        .map(|v| serde_json::from_slice::<Mimic>(&v))
        .transpose()?)
}

// ---- Slash command group: /mimic ----

#[poise::command(
    slash_command,
    subcommands("mimic_add", "mimic_list", "mimic_delete", "mimic_set", "mimic_say"),
    guild_only
)]
pub async fn mimic(_ctx: Ctx<'_>) -> Result<()> {
    Ok(())
}

/// /mimic add — create a mimic from either an avatar+name, or copy from a user
#[poise::command(slash_command, guild_only)]
pub async fn mimic_add(
    ctx: Ctx<'_>,

    #[description = "Name for this mimic (ignored if copying and you leave this empty)"]
    name: Option<String>,
    #[description = "Avatar URL (optional if copying from a user)"] avatar_url: Option<String>,
    #[description = "Attachment avatar (optional; overrides URL if given)"] attachment: Option<
        serenity::Attachment,
    >,
    #[description = "Copy name & avatar from this user (optional)"] copy_from: Option<
        serenity::User,
    >,
) -> Result<()> {
    let guild = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("Use /mimic in a server"))?;
    let user = ctx.author().id;

    // Decide name + avatar
    let (final_name, final_avatar): (String, Option<String>) = if let Some(u) = copy_from {
        let default_name = u.name.clone();
        let chosen_name = name.unwrap_or(default_name);

        // prefer custom avatar; fallback to default avatar URL
        let mut url = u.avatar_url();
        if url.is_none() {
            url = Some(u.default_avatar_url());
        }
        // If caller supplied avatar URL/attachment, override the copied one
        let att_url = attachment.as_ref().map(|a| a.url.clone());
        let chosen_avatar = att_url.or(avatar_url).or(url);

        (chosen_name, chosen_avatar)
    } else {
        let chosen_name = name.ok_or_else(|| anyhow!("Provide a name or use copy_from"))?;
        let att_url = attachment.as_ref().map(|a| a.url.clone());
        let chosen_avatar = att_url.or(avatar_url);
        (chosen_name, chosen_avatar)
    };

    let m = Mimic {
        name: final_name.clone(),
        avatar_url: final_avatar,
    };
    put_mimic(&ctx.data().db, guild.0, user.0, &m)?;
    ctx.say(format!("✅ Added mimic **{final_name}**")).await?;
    Ok(())
}

/// /mimic list — shows all mimics, marks the active one
#[poise::command(slash_command, guild_only)]
pub async fn mimic_list(ctx: Ctx<'_>) -> Result<()> {
    let guild = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("Use /mimic in a server"))?;
    let user = ctx.author().id;

    let list = list_mimics(&ctx.data().db, guild.0, user.0)?;

    let active = get_active(&ctx.data().db, guild.0, user.0)?;

    if list.is_empty() {
        ctx.say("You have no mimics yet. Use `/mimic add`.").await?;

        return Ok(());
    }

    let mut out = String::new();
    for m in list {
        let star = active
            .as_deref()
            .map(|a| a.eq_ignore_ascii_case(&m.name))
            .unwrap_or(false);
        let mark = if star { "★" } else { "•" };
        let avatar = m.avatar_url.as_deref().unwrap_or("none");
        out.push_str(&format!("{mark} **{}** — avatar: {avatar}\n", m.name));
    }
    ctx.say(out).await?;
    Ok(())
}

/// /mimic delete — remove a mimic
#[poise::command(slash_command, guild_only)]
pub async fn mimic_delete(
    ctx: Ctx<'_>,
    #[description = "Mimic name to delete"] name: String,
) -> Result<()> {
    let guild = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("Use /mimic in a server"))?;
    let user = ctx.author().id;

    if del_mimic(&ctx.data().db, guild.0, user.0, &name)? {
        ctx.say("🗑️ Deleted.").await?;
    } else {
        ctx.say("Not found.").await?;
    }
    Ok(())
}

/// /mimic set — sets your active mimic
#[poise::command(slash_command, guild_only)]
pub async fn mimic_set(
    ctx: Ctx<'_>,

    #[description = "Mimic name to activate"] name: String,
) -> Result<()> {
    let guild = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("Use /mimic in a server"))?;
    let user = ctx.author().id;

    set_active(&ctx.data().db, guild.0, user.0, &name)?;
    ctx.say(format!("⭐ Active mimic set to **{name}**"))
        .await?;

    Ok(())
}

/// /mimic say — speak as your active mimic in this channel

#[poise::command(slash_command, guild_only)]
pub async fn mimic_say(
    ctx: Ctx<'_>,
    #[description = "What should your mimic say?"] text: String,
) -> Result<()> {
    let guild = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("Use /mimic in a server"))?;
    let user = ctx.author().id;

    let Some(active_name) = get_active(&ctx.data().db, guild.0, user.0)? else {
        ctx.say("Set an active mimic first with `/mimic set`.")
            .await?;
        return Ok(());
    };
    let m = get_mimic(&ctx.data().db, guild.0, user.0, &active_name)?
        .ok_or_else(|| anyhow!("Active mimic not found (was it deleted)?"))?;

    // Create/find webhook in this channel
    let hook = get_or_create_webhook(&ctx.serenity_context().http, ctx.channel_id()).await?;

    // Send through the webhook with the mimic's name + avatar
    let exec = serenity::ExecuteWebhook::new()
        .content(text)
        .username(&m.name);
    let exec = if let Some(url) = &m.avatar_url {
        exec.avatar_url(url)
    } else {
        exec
    };

    hook.execute(&ctx.serenity_context().http, false, exec)
        .await
        .context("Webhook execute failed (need Manage Webhooks)")?;

    Ok(())
}
