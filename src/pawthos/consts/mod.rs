use poise::serenity_prelude as serenity;
use serenity::Color;
use serenity::GatewayIntents;

pub const INTENTS: GatewayIntents = {
    let mut r = GatewayIntents::GUILD_MESSAGES;
    r = GatewayIntents::union(r, GatewayIntents::DIRECT_MESSAGES);
    r = GatewayIntents::union(r, GatewayIntents::MESSAGE_CONTENT);
    r
};

pub const LOGOS_GREEN: Color = Color::from_rgb(102, 204, 102);
pub const LOGOS_RED: Color = Color::from_rgb(255, 0, 0);
