#[derive(Debug)]
pub enum ScheduleError {
    NoUserFound,
    EventNotFound,
    ParseError(chrono::ParseError),
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScheduleError::NoUserFound => write!(f, "This user does not have a Schedule!"),
            ScheduleError::EventNotFound => write!(f, "Could not find that event!"),
            ScheduleError::ParseError(e) => write!(f, "{}", e),
        }
    }
}

impl From<chrono::ParseError> for ScheduleError {
    fn from(value: chrono::ParseError) -> Self {
        ScheduleError::ParseError(value)
    }
}

impl std::error::Error for ScheduleError {}
