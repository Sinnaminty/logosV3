use clap::Parser;
use log::LevelFilter;
use poise::serenity_prelude as serenity;
use utils::ResultExt;
mod commands;
mod dectalk;
mod framework;
mod handlers;
mod logging;
mod setup;
mod types;
mod utils;

/// V3 of the same bot. I need a job...
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value_t = LevelFilter::Warn)]
    pub log_level: LevelFilter,
}

#[tokio::main]
async fn main() {
    setup::setup_logging(Args::parse().log_level);

    //FIXME: change this maybe? i'd like to obscure this setup.

    let framework = setup::setup_framework();

    let mut client = serenity::ClientBuilder::new(setup::get_api_token(), setup::INTENTS)
        .framework(framework)
        .await
        .unwrap_or_log();

    // lovely jubly!
    client.start().await.unwrap_or_log();
}
