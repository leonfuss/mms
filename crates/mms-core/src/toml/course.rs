use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::Result;

/// Contents of a `.course.toml` file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseToml {
    /// Short name/code of the course (e.g., "cs101")
    pub short_name: String,

    /// Full course name
    pub name: String,

    /// ECTS credits
    pub ects: i32,

    /// Lecturer name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lecturer: Option<String>,

    /// Lecturer email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lecturer_email: Option<String>,

    /// Tutor name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tutor: Option<String>,

    /// Tutor email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tutor_email: Option<String>,

    /// Learning platform URL (e.g., Moodle)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_platform_url: Option<String>,

    /// University (overrides semester default if set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub university: Option<String>,

    /// Location (overrides semester default if set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Whether this course is external (imported from elsewhere)
    #[serde(default)]
    pub is_external: bool,

    /// Original path if external
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_path: Option<String>,

    /// Whether course is dropped
    #[serde(default)]
    pub is_dropped: bool,

    /// Whether this course has a git repository
    #[serde(default)]
    pub has_git_repo: bool,

    /// Git remote URL if repository exists
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_remote_url: Option<String>,

    /// Course-specific metadata (for custom extensions)
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl CourseToml {
    /// Create a new course TOML with required fields
    pub fn new(short_name: String, name: String, ects: i32) -> Self {
        Self {
            short_name,
            name,
            ects,
            lecturer: None,
            lecturer_email: None,
            tutor: None,
            tutor_email: None,
            learning_platform_url: None,
            university: None,
            location: None,
            is_external: false,
            original_path: None,
            is_dropped: false,
            has_git_repo: false,
            git_remote_url: None,
            metadata: None,
        }
    }

    /// Read a course TOML file from disk
    pub fn read(path: &Path) -> Result<Self> {
        super::read_toml_file(path)
    }

    /// Write this course TOML to disk
    pub fn write(&self, path: &Path) -> Result<()> {
        super::write_toml_file(path, self)
    }

    /// Read from a course directory (looks for .course.toml)
    pub fn read_from_directory(course_dir: &Path) -> Result<Self> {
        let toml_path = course_dir.join(".course.toml");
        Self::read(&toml_path)
    }

    /// Write to a course directory (creates .course.toml)
    pub fn write_to_directory(&self, course_dir: &Path) -> Result<()> {
        let toml_path = course_dir.join(".course.toml");
        self.write(&toml_path)
    }

    /// Get the expected path for this course's TOML file
    pub fn toml_path(&self, semester_dir: &Path) -> PathBuf {
        semester_dir.join(&self.short_name).join(".course.toml")
    }

    // Builder methods for convenient construction

    pub fn with_lecturer(mut self, lecturer: String) -> Self {
        self.lecturer = Some(lecturer);
        self
    }

    pub fn with_lecturer_email(mut self, email: String) -> Self {
        self.lecturer_email = Some(email);
        self
    }

    pub fn with_tutor(mut self, tutor: String) -> Self {
        self.tutor = Some(tutor);
        self
    }

    pub fn with_tutor_email(mut self, email: String) -> Self {
        self.tutor_email = Some(email);
        self
    }

    pub fn with_learning_platform_url(mut self, url: String) -> Self {
        self.learning_platform_url = Some(url);
        self
    }

    pub fn with_university(mut self, university: String) -> Self {
        self.university = Some(university);
        self
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_external(mut self, is_external: bool) -> Self {
        self.is_external = is_external;
        self
    }

    pub fn with_original_path(mut self, path: String) -> Self {
        self.original_path = Some(path);
        self
    }

    pub fn with_dropped(mut self, is_dropped: bool) -> Self {
        self.is_dropped = is_dropped;
        self
    }

    pub fn with_git_repo(mut self, remote_url: String) -> Self {
        self.has_git_repo = true;
        self.git_remote_url = Some(remote_url);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_course_new() {
        let course = CourseToml::new(
            "cs101".to_string(),
            "Introduction to Algorithms".to_string(),
            8,
        );

        assert_eq!(course.short_name, "cs101");
        assert_eq!(course.name, "Introduction to Algorithms");
        assert_eq!(course.ects, 8);
        assert_eq!(course.lecturer, None);
        assert_eq!(course.is_external, false);
    }

    #[test]
    fn test_course_builder() {
        let course = CourseToml::new(
            "cs101".to_string(),
            "Introduction to Algorithms".to_string(),
            8,
        )
        .with_lecturer("Prof. Dr. Schmidt".to_string())
        .with_lecturer_email("schmidt@tum.de".to_string())
        .with_tutor("Anna Müller".to_string())
        .with_university("TUM".to_string())
        .with_git_repo("https://github.com/user/cs101".to_string());

        assert_eq!(course.lecturer, Some("Prof. Dr. Schmidt".to_string()));
        assert_eq!(course.lecturer_email, Some("schmidt@tum.de".to_string()));
        assert_eq!(course.tutor, Some("Anna Müller".to_string()));
        assert_eq!(course.university, Some("TUM".to_string()));
        assert_eq!(course.has_git_repo, true);
        assert_eq!(course.git_remote_url, Some("https://github.com/user/cs101".to_string()));
    }

    #[test]
    fn test_serialize_deserialize() {
        let course = CourseToml::new(
            "math201".to_string(),
            "Linear Algebra".to_string(),
            6,
        )
        .with_lecturer("Prof. Wagner".to_string())
        .with_university("TUM".to_string());

        let toml_str = toml::to_string(&course).unwrap();
        let deserialized: CourseToml = toml::from_str(&toml_str).unwrap();

        assert_eq!(deserialized.short_name, course.short_name);
        assert_eq!(deserialized.name, course.name);
        assert_eq!(deserialized.ects, course.ects);
        assert_eq!(deserialized.lecturer, course.lecturer);
        assert_eq!(deserialized.university, course.university);
    }

    #[test]
    fn test_toml_path() {
        let course = CourseToml::new(
            "cs101".to_string(),
            "Intro to Algorithms".to_string(),
            8,
        );

        let semester_dir = PathBuf::from("/Users/test/Studies/b3");
        let expected_path = PathBuf::from("/Users/test/Studies/b3/cs101/.course.toml");

        assert_eq!(course.toml_path(&semester_dir), expected_path);
    }
}
