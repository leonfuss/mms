use thiserror::Error;

#[derive(Error, Debug)]
pub enum MmsError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Date/time parse error: {0}")]
    ChronoParse(#[from] chrono::ParseError),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Course not found: {0}")]
    CourseNotFound(i64),

    #[error("Semester not found: {0}")]
    SemesterNotFound(i64),

    #[error("Schedule not found: {0}")]
    ScheduleNotFound(i64),

    #[error("Todo not found: {0}")]
    TodoNotFound(i64),

    #[error("Holiday not found: {0}")]
    HolidayNotFound(i64),

    #[error("No active course at this time")]
    NoActiveCourse,

    #[error("Invalid schedule: {0}")]
    InvalidSchedule(String),

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Invalid time format: {0}")]
    InvalidTime(String),

    #[error("Invalid semester type: {0}")]
    InvalidSemesterType(String),

    #[error("Invalid schedule type: {0}")]
    InvalidScheduleType(String),

    #[error("Invalid exam type: {0}")]
    InvalidExamType(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, MmsError>;
