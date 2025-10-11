use poise::serenity_prelude::{self as serenity, GatewayIntents};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mimic {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimicDB {
    db: HashMap<serenity::UserId, Vec<Mimic>>,
}

#[derive(Debug)]
pub struct Data {
    pub mimic_db: MimicDB,
} // User data, which is stored and accessible in all command invocations

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Embed = serenity::builder::CreateEmbed;
pub type Reply = poise::reply::CreateReply;
pub type Result<T = ()> = std::result::Result<T, Error>;

pub enum EmbedType {
    Good,
    Bad,
    Neutral,
}

impl EmbedType {
    pub fn into_color(self) -> serenity::Color {
        match self {
            EmbedType::Good => LOGOS_GREEN,
            EmbedType::Bad => LOGOS_RED,
            EmbedType::Neutral => serenity::Color::FABLED_PINK,
        }
    }
}

pub const INTENTS: GatewayIntents = {
    let mut r = GatewayIntents::GUILD_MESSAGES;
    r = GatewayIntents::union(r, GatewayIntents::DIRECT_MESSAGES);
    r = GatewayIntents::union(r, GatewayIntents::MESSAGE_CONTENT);
    r
};
pub const LOGOS_GREEN: serenity::Color = serenity::Color::from_rgb(102, 204, 102);
pub const LOGOS_RED: serenity::Color = serenity::Color::from_rgb(255, 0, 0);
