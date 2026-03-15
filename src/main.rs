//! Entry point for logosV3.
//!
//! Parses CLI arguments, initialises logging, builds the Poise/Serenity client,
//! and starts the bot. The only startup I/O this module performs is reading the
//! log-level flag; everything else is delegated to [`setup`].

use clap::Parser;
use log::LevelFilter;
use poise::serenity_prelude as serenity;
use utils::ResultExt;
mod commands;
mod dectalk;
mod framework;
mod handlers;
mod logging;
mod pawthos;
mod setup;
mod utils;

/// Command-line arguments for logosV3.
///
/// Pass `--log-level debug` (or `-l debug`) for verbose output during
/// development. Defaults to `info` in production.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value_t = LevelFilter::Info)]
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
        .unwrap_or_log("main::client");

    // lovely jubly!
    client.start().await.unwrap_or_log("main::start");
}
