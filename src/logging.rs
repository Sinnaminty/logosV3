//! Logging initialisation for logosV3.
//!
//! Wraps [`simple_logger`] with per-module overrides that silence noisy
//! third-party crates at the configured level. Serenity's HTTP and gateway
//! layers, as well as `tracing::span`, are capped at `WARN` regardless of the
//! global level so that the logs stay readable at `DEBUG` or `TRACE`.

use log::LevelFilter;
use simple_logger::SimpleLogger;

/// Initialise global logging.
///
/// Sets the global log level to `l`, then overrides specific noisy modules:
/// - `tracing::span` → `WARN`
/// - `serenity::http` → `WARN`
/// - `serenity::gateway` → `WARN`
///
/// # Panics
/// Panics if the logger has already been initialised (which should never happen
/// in normal operation since this is called exactly once at startup).
pub fn setup_logging(l: LevelFilter) {
    SimpleLogger::new()
        .with_level(l)
        .with_module_level("tracing::span", LevelFilter::Warn)
        .with_module_level("serenity::http", LevelFilter::Warn)
        .with_module_level("serenity::gateway", LevelFilter::Warn)
        .init()
        .expect("Failed to set up logging. Panic!");
}
