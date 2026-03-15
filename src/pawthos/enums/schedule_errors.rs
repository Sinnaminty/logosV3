//! Error type for the schedule sub-system.

/// Errors that can occur when working with a user's schedule.
#[derive(thiserror::Error, Debug)]
pub enum ScheduleError {
    /// The calling user has no entry in the schedule database.
    ///
    /// This is a normal state for new users who have never used `/schedule add`.
    #[error("This user does not have a Schedule!")]
    NoUserFound,

    /// The requested event name was not found in the user's event list.
    #[error("Could not find that event!")]
    EventNotFound,

    /// A date or time string supplied by the user could not be parsed.
    ///
    /// The inner [`chrono::ParseError`] describes the parse failure.
    /// The `#[from]` attribute auto-generates `From<chrono::ParseError>` for
    /// this variant.
    #[error(transparent)]
    ParseError(#[from] chrono::ParseError),

    /// The supplied timezone string was not recognised by `chrono_tz`.
    #[error("Unknown timezone: {0}")]
    InvalidTimezone(String),

    /// The supplied date/time falls in a DST gap or is otherwise ambiguous,
    /// so it cannot be unambiguously localised to the user's timezone.
    #[error("That time is ambiguous or invalid (e.g. falls in a DST gap). Try a different time.")]
    AmbiguousOrInvalidTime,
}
