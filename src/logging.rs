use log::LevelFilter;
use simple_logger::SimpleLogger;

pub fn setup_logging(l: LevelFilter) {
    SimpleLogger::new()
        .with_level(l)
        .with_module_level("tracing::span", LevelFilter::Warn)
        .with_module_level("serenity::http", LevelFilter::Warn)
        .with_module_level("serenity::gateway", LevelFilter::Warn)
        .init()
        .expect("Failed to set up logging. Panic!");
}
