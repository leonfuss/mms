use thiserror::Error;

#[derive(Error, Debug)]
pub enum MmsError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Invalid base path: {0}")]
    BasePathInvalid(String),

    #[error("Config file not found at path: {path}")]
    ConfigNotFound { path: std::path::PathBuf },

    #[error("Failed to parse config file at {path}: {source}")]
    ConfigParseError {
        path: std::path::PathBuf,
        source: toml::de::Error,
    },

    #[error("Failed to serialize config to {path}: {source}")]
    ConfigSerializeError {
        path: std::path::PathBuf,
        source: toml::ser::Error,
    },

    #[error("Missing required configuration field: university_base_path")]
    UniversityBasePathMissing,

    #[error("University base path parent directory does not exist: {parent} (from path: {path})")]
    UniversityBasePathParentNotFound {
        path: std::path::PathBuf,
        parent: std::path::PathBuf,
    },

    #[error("Could not determine config directory")]
    ConfigDirNotFound,

    #[error("Could not determine data directory")]
    DataDirNotFound,

    #[error("Symlink path not configured in general settings")]
    SymlinkNotSet,

    #[error("Schedule configuration not set")]
    ScheduleNotSet,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Date/time parse error: {0}")]
    ChronoParse(#[from] chrono::ParseError),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

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