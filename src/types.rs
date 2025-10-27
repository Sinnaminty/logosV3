use poise::serenity_prelude::{self as serenity, ChannelId, GatewayIntents, UserId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;

use crate::{dectalk::DectalkError, types};

#[derive(Debug)]
pub enum PersistantData {
    MimicDB(MimicDB),
}

#[derive(thiserror::Error, Debug)]
pub enum LogosErrors {
    #[error("SerenityError: {0}")]
    Serenity(#[from] poise::serenity_prelude::Error),

    #[error("ffiError: {0}")]
    FfiNul(#[from] std::ffi::NulError),

    #[error("tokio::JoinError: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),

    #[error("tokio::SendError {0}")]
    TokioSend(#[from] tokio::sync::mpsc::error::SendError<types::PersistantData>),

    #[error("DectalkError: {0}")]
    Dectalk(#[from] DectalkError),

    #[error("std::io: {0}")]
    StdIo(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mimic {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MimicUser {
    pub active_mimic: Option<Mimic>,
    pub mimics: Vec<Mimic>,
    pub auto_mode: Option<bool>,
    #[serde(default)]
    pub channel_override: HashMap<ChannelId, Mimic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MimicDB {
    db: HashMap<serenity::UserId, MimicUser>,
}

impl MimicDB {
    /// returns the MimicUser stored inside of the Db. will create a new MimicUser entry if the
    /// userID is not found inside of the Db.
    pub fn get_user(&mut self, user: UserId) -> &mut MimicUser {
        self.db.entry(user).or_default()
    }
}

#[derive(Debug)]
pub struct Data {
    pub mimic_db: Mutex<MimicDB>,
    pub persistant_data_channel: tokio::sync::mpsc::Sender<PersistantData>,
} // User data, which is stored and accessible in all command invocations

pub type Error = LogosErrors;
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
