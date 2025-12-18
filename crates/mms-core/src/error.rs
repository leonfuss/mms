use thiserror::Error;

#[derive(Debug, Clone)]
pub struct MissingConfigFields {
    pub student_name: bool,
    pub student_id: bool,
    pub studies_root: bool,
    pub default_editor: bool,
    pub default_pdf_viewer: bool,
    pub default_location: bool,
}

impl MissingConfigFields {
    pub fn new() -> Self {
        Self {
            student_name: false,
            student_id: false,
            studies_root: false,
            default_editor: false,
            default_pdf_viewer: false,
            default_location: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.student_name
            && !self.student_id
            && !self.studies_root
            && !self.default_editor
            && !self.default_pdf_viewer
            && !self.default_location
    }
}

impl std::fmt::Display for MissingConfigFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut missing = Vec::new();
        if self.student_name { missing.push("student_name"); }
        if self.student_id { missing.push("student_id"); }
        if self.studies_root { missing.push("studies_root"); }
        if self.default_editor { missing.push("default_editor"); }
        if self.default_pdf_viewer { missing.push("default_pdf_viewer"); }
        if self.default_location { missing.push("default_location"); }
        write!(f, "{}", missing.join(", "))
    }
}

#[derive(Error, Debug)]
pub enum MmsError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Configuration is incomplete. Missing required fields: {0}")]
    ConfigIncomplete(MissingConfigFields),

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