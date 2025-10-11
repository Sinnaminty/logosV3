use crate::types::{Mimic, MimicDB, Result};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Webhook;
use serenity::GuildId;
use serenity::UserId;

impl MimicDB {
    pub fn put_mimic(user: UserId, m: &Mimic) -> Result {
        todo!();
    }
}
pub async fn get_or_create_webhook(
    http: &serenity::Http,
    channel_id: serenity::ChannelId,
) -> Result<Webhook> {
    const WEBHOOK_NAME: &str = "logosV3-mimic";
    if let Ok(existing) = channel_id.webhooks(http).await {
        if let Some(w) = existing
            .into_iter()
            .find(|w| w.name.as_deref() == Some(WEBHOOK_NAME))
        {
            return Ok(w);
        }
    }

    //the webby don't exist :c
    let hook = channel_id
        .create_webhook(http, serenity::CreateWebhook::new(WEBHOOK_NAME))
        .await?;
    Ok(hook)
}

pub fn put_mimic(user: UserId, m: &Mimic) -> Result {
    let val = serde_json::to_vec(m)?;
    mimic_db.insert(key.as_bytes(), val)?;
    mimic_db.flush()?;
    Ok(())
}

pub fn del_mimic(mimic_db: &mimic_db, guild: GuildId, user: UserId, name: &str) -> Result {
    let key = Mimic::key(guild, user, name);
    let existed = mimic_db.remove(key.as_bytes())?.is_some();
    if existed {
        // if it was active, clear it
        let ak = active_key(guild, user);
        if let Some(cur) = mimic_db.get(ak.as_bytes())? {
            let cur_name: String = serde_json::from_slice(&cur)?;
            if cur_name.eq_ignore_ascii_case(name) {
                let _ = mimic_db.remove(ak.as_bytes())?;
            }
        }
        mimic_db.flush()?;
    }
    Ok(existed)
}

pub fn list_mimics(mimic_db: &mimic_db, guild: GuildId, user: UserId) -> Result {
    let prefix = format!("mimic/{guild}/{user}/");
    let mut v = Vec::new();
    for item in mimic_db.scan_prefix(prefix.as_bytes()) {
        let (_, bytes) = item?;
        v.push(serde_json::from_slice::<Mimic>(&bytes)?);
    }
    Ok(v)
}

pub fn set_active(mimic_db: &mimic_db, guild: GuildId, user: UserId, name: &str) -> Result {
    // ensure exists
    let key = Mimic::key(guild, user, name);
    if mimic_db.get(key.as_bytes())?.is_none() {
        return Err(anyhow!("No mimic named `{name}` found"));
    }
    let ak = active_key(guild, user);
    mimic_db.insert(ak.as_bytes(), serde_json::to_vec(&name)?)?;
    mimic_db.flush()?;
    Ok(())
}

pub fn get_active(mimic_db: &mimic_db, guild: GuildId, user: UserId) -> Result {
    let ak = active_key(guild, user);
    Ok(mimic_db
        .get(ak.as_bytes())?
        .map(|v| serde_json::from_slice::<String>(&v))
        .transpose()?)
}

pub fn get_mimic(mimic_db: &mimic_db, guild: GuildId, user: UserId, name: &str) -> Result {
    let key = Mimic::key(guild, user, name);
    Ok(mimic_db
        .get(key.as_bytes())?
        .map(|v| serde_json::from_slice::<Mimic>(&v))
        .transpose()?)
}
