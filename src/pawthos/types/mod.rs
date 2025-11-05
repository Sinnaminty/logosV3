use crate::pawthos::enums::logos_errors::LogosErrors;
use crate::pawthos::structs::data::Data;
use poise::serenity_prelude as serenity;

pub type Error = LogosErrors;

pub type Context<'a> = poise::Context<'a, Data, Error>;

pub type Embed = serenity::builder::CreateEmbed;

pub type Reply = poise::reply::CreateReply;

pub type Result<T = (), E = Error> = std::result::Result<T, E>;
