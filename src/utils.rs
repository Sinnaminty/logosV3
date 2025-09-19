use clap::Parser;
use log::LevelFilter;
use poise::serenity_prelude as serenity;
use serenity::all::GatewayIntents;
use simple_logger::SimpleLogger;

/// this is a trait!!
pub trait ResultExt<T, E> {
    /// Unwraps the result, logging the error and panicking if it's an Err.
    fn unwrap_or_log(self) -> T;
}

impl<T, E: std::fmt::Display> ResultExt<T, E> for Result<T, E> {
    /// this is the func :o
    fn unwrap_or_log(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                log::error!("Unrecoverable error: {e}");
                panic!();
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct APIKey {
    token: String,
}

/// V3 of the same bot. I need a job...
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,
}

pub fn setup_logging(l: LevelFilter) {
    SimpleLogger::new()
        .with_level(l)
        .with_module_level("tracing::span", LevelFilter::Warn)
        .with_module_level("serenity::http", LevelFilter::Warn)
        .init()
        .expect("Failed to set up logging. Panic!");
}

pub const INTENTS: GatewayIntents = {
    let mut r = GatewayIntents::GUILD_MESSAGES;
    r = GatewayIntents::union(r, GatewayIntents::DIRECT_MESSAGES);
    r = GatewayIntents::union(r, GatewayIntents::MESSAGE_CONTENT);
    r
};

pub fn get_api_token() -> String {
    let file_contents = std::fs::read_to_string("s.json").unwrap_or_log();
    let api_key: APIKey = serenity::json::from_str(file_contents).unwrap_or_log();
    api_key.token
}
