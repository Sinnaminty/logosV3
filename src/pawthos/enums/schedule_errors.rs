#[derive(thiserror::Error, Debug)]
pub enum ScheduleError {
    #[error("This user does not have a Schedule!")]
    NoUserFound,
    #[error("Could not find that event!")]
    EventNotFound,
    #[error(transparent)]
    ParseError(#[from] chrono::ParseError),
    #[error("Unknown timezone: {0}")]
    InvalidTimezone(String),
    #[error("That time is ambiguous or invalid (e.g. falls in a DST gap). Try a different time.")]
    AmbiguousOrInvalidTime,
}
