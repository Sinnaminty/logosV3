use crate::types::{Mimic, MimicDB, MimicUser, Result};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::Webhook;
use serenity::UserId;

impl MimicDB {
    /// returns the MimicUser stored inside of the Db. will create a new MimicUser entry if the
    /// userID is not found inside of the Db.
    pub fn get_user(&self, user: UserId) -> MimicUser {
        todo!();
    }
}

impl MimicUser {
    /// adds this Mimic to the mimics member variable of this user's MimicUser struct.
    pub fn add_mimic(&self, mimic: &Mimic) {
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
