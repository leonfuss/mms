use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::Result;

/// Semester type (Bachelor or Master)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SemesterType {
    Bachelor,
    Master,
}

impl SemesterType {
    /// Get the short code prefix (b or m)
    pub fn prefix(&self) -> char {
        match self {
            SemesterType::Bachelor => 'b',
            SemesterType::Master => 'm',
        }
    }

    /// Parse from a string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bachelor" | "b" => Some(SemesterType::Bachelor),
            "master" | "m" => Some(SemesterType::Master),
            _ => None,
        }
    }
}

impl std::fmt::Display for SemesterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemesterType::Bachelor => write!(f, "bachelor"),
            SemesterType::Master => write!(f, "master"),
        }
    }
}

/// Contents of a `.semester.toml` file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemesterToml {
    /// Semester type (bachelor or master)
    #[serde(rename = "type")]
    pub semester_type: SemesterType,

    /// Semester number (e.g., 1, 2, 3...)
    pub number: i32,

    /// Start date in ISO format (YYYY-MM-DD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,

    /// End date in ISO format (YYYY-MM-DD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,

    /// Default university for courses in this semester
    #[serde(skip_serializing_if = "Option::is_none")]
    pub university: Option<String>,

    /// Default location for courses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Whether this semester is currently active
    #[serde(default)]
    pub is_current: bool,

    /// Whether this semester is archived
    #[serde(default)]
    pub is_archived: bool,
}

impl SemesterToml {
    /// Create a new semester TOML with required fields
    pub fn new(semester_type: SemesterType, number: i32) -> Self {
        Self {
            semester_type,
            number,
            start_date: None,
            end_date: None,
            university: None,
            location: None,
            is_current: false,
            is_archived: false,
        }
    }

    /// Get the semester code (e.g., "b1", "m2")
    pub fn code(&self) -> String {
        format!("{}{}", self.semester_type.prefix(), self.number)
    }

    /// Read a semester TOML file from disk
    pub fn read(path: &Path) -> Result<Self> {
        super::read_toml_file(path)
    }

    /// Write this semester TOML to disk
    pub fn write(&self, path: &Path) -> Result<()> {
        super::write_toml_file(path, self)
    }

    /// Read from a semester directory (looks for .semester.toml)
    pub fn read_from_directory(semester_dir: &Path) -> Result<Self> {
        let toml_path = semester_dir.join(".semester.toml");
        Self::read(&toml_path)
    }

    /// Write to a semester directory (creates .semester.toml)
    pub fn write_to_directory(&self, semester_dir: &Path) -> Result<()> {
        let toml_path = semester_dir.join(".semester.toml");
        self.write(&toml_path)
    }

    /// Get the expected path for this semester's TOML file
    pub fn toml_path(&self, base_path: &Path) -> PathBuf {
        base_path.join(self.code()).join(".semester.toml")
    }

    /// Builder method to set start date
    pub fn with_start_date(mut self, date: String) -> Self {
        self.start_date = Some(date);
        self
    }

    /// Builder method to set end date
    pub fn with_end_date(mut self, date: String) -> Self {
        self.end_date = Some(date);
        self
    }

    /// Builder method to set university
    pub fn with_university(mut self, university: String) -> Self {
        self.university = Some(university);
        self
    }

    /// Builder method to set location
    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    /// Builder method to mark as current
    pub fn with_current(mut self, is_current: bool) -> Self {
        self.is_current = is_current;
        self
    }

    /// Builder method to mark as archived
    pub fn with_archived(mut self, is_archived: bool) -> Self {
        self.is_archived = is_archived;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semester_type_prefix() {
        assert_eq!(SemesterType::Bachelor.prefix(), 'b');
        assert_eq!(SemesterType::Master.prefix(), 'm');
    }

    #[test]
    fn test_semester_type_from_str() {
        assert_eq!(SemesterType::from_str("bachelor"), Some(SemesterType::Bachelor));
        assert_eq!(SemesterType::from_str("Bachelor"), Some(SemesterType::Bachelor));
        assert_eq!(SemesterType::from_str("b"), Some(SemesterType::Bachelor));
        assert_eq!(SemesterType::from_str("master"), Some(SemesterType::Master));
        assert_eq!(SemesterType::from_str("m"), Some(SemesterType::Master));
        assert_eq!(SemesterType::from_str("invalid"), None);
    }

    #[test]
    fn test_semester_code() {
        let sem = SemesterToml::new(SemesterType::Bachelor, 1);
        assert_eq!(sem.code(), "b1");

        let sem = SemesterToml::new(SemesterType::Master, 5);
        assert_eq!(sem.code(), "m5");
    }

    #[test]
    fn test_semester_builder() {
        let sem = SemesterToml::new(SemesterType::Bachelor, 3)
            .with_start_date("2024-10-01".to_string())
            .with_end_date("2025-03-31".to_string())
            .with_university("TUM".to_string())
            .with_current(true);

        assert_eq!(sem.semester_type, SemesterType::Bachelor);
        assert_eq!(sem.number, 3);
        assert_eq!(sem.start_date, Some("2024-10-01".to_string()));
        assert_eq!(sem.end_date, Some("2025-03-31".to_string()));
        assert_eq!(sem.university, Some("TUM".to_string()));
        assert_eq!(sem.is_current, true);
        assert_eq!(sem.is_archived, false);
    }

    #[test]
    fn test_serialize_deserialize() {
        let sem = SemesterToml::new(SemesterType::Bachelor, 1)
            .with_start_date("2024-10-01".to_string())
            .with_university("TUM".to_string());

        let toml_str = toml::to_string(&sem).unwrap();
        let deserialized: SemesterToml = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.semester_type, sem.semester_type);
        assert_eq!(deserialized.number, sem.number);
        assert_eq!(deserialized.start_date, sem.start_date);
        assert_eq!(deserialized.university, sem.university);
    }
}
