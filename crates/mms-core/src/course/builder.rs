use crate::course::operations::{CourseInfo, create_course};
use crate::error::Result;
use sea_orm::DatabaseConnection;

/// Builder for creating a new course
///
/// # Example
/// ```no_run
/// use mms_core::course::CourseBuilder;
/// use mms_core::config::Config;
/// use sea_orm::DatabaseConnection;
///
/// # async fn example(config: &Config, db: &DatabaseConnection) -> mms_core::error::Result<()> {
/// let course = CourseBuilder::new("cs101", "Introduction to Algorithms", 8)
///     .in_semester(1) // semester_id from database
///     .with_lecturer("Prof. Dr. Schmidt")
///     .with_lecturer_email("schmidt@tum.de")
///     .with_tutor("Anna Müller")
///     .with_university("TUM")
///     .create(db)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct CourseBuilder {
    short_name: String,
    name: String,
    ects: i32,
    semester_id: Option<i64>,
    lecturer: Option<String>,
    lecturer_email: Option<String>,
    tutor: Option<String>,
    tutor_email: Option<String>,
    learning_platform_url: Option<String>,
    university: Option<String>,
    location: Option<String>,
    is_external: bool,
    original_path: Option<String>,
    has_git_repo: bool,
    git_remote_url: Option<String>,
}

impl CourseBuilder {
    /// Create a new course builder with required fields
    ///
    /// # Arguments
    /// * `short_name` - Course code (e.g., "cs101")
    /// * `name` - Full course name
    /// * `ects` - ECTS credits
    pub fn new<S1, S2>(short_name: S1, name: S2, ects: i32) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            short_name: short_name.into(),
            name: name.into(),
            ects,
            semester_id: None,
            lecturer: None,
            lecturer_email: None,
            tutor: None,
            tutor_email: None,
            learning_platform_url: None,
            university: None,
            location: None,
            is_external: false,
            original_path: None,
            has_git_repo: false,
            git_remote_url: None,
        }
    }

    /// Set the semester this course belongs to (required for creation)
    pub fn in_semester(mut self, semester_id: i64) -> Self {
        self.semester_id = Some(semester_id);
        self
    }

    /// Set the lecturer name
    pub fn with_lecturer<S: Into<String>>(mut self, lecturer: S) -> Self {
        self.lecturer = Some(lecturer.into());
        self
    }

    /// Set the lecturer email
    pub fn with_lecturer_email<S: Into<String>>(mut self, email: S) -> Self {
        self.lecturer_email = Some(email.into());
        self
    }

    /// Set the tutor name
    pub fn with_tutor<S: Into<String>>(mut self, tutor: S) -> Self {
        self.tutor = Some(tutor.into());
        self
    }

    /// Set the tutor email
    pub fn with_tutor_email<S: Into<String>>(mut self, email: S) -> Self {
        self.tutor_email = Some(email.into());
        self
    }

    /// Set the learning platform URL (e.g., Moodle)
    pub fn with_learning_platform_url<S: Into<String>>(mut self, url: S) -> Self {
        self.learning_platform_url = Some(url.into());
        self
    }

    /// Set the university (overrides semester default)
    pub fn with_university<S: Into<String>>(mut self, university: S) -> Self {
        self.university = Some(university.into());
        self
    }

    /// Set the location (overrides semester default)
    pub fn with_location<S: Into<String>>(mut self, location: S) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Mark this course as external (imported from elsewhere)
    pub fn as_external(mut self, original_path: Option<String>) -> Self {
        self.is_external = true;
        self.original_path = original_path;
        self
    }

    /// Set git repository information
    pub fn with_git_repo<S: Into<String>>(mut self, remote_url: S) -> Self {
        self.has_git_repo = true;
        self.git_remote_url = Some(remote_url.into());
        self
    }

    /// Create the course (folder + TOML + database entry)
    ///
    /// This method will:
    /// 1. Validate that semester_id is set
    /// 2. Get semester directory path
    /// 3. Create the course directory (e.g., ~/Studies/b3/cs101/)
    /// 4. Write the .course.toml file
    /// 5. Create the database entry
    ///
    /// Returns the created course info
    pub async fn create(self, db: &DatabaseConnection) -> Result<CourseInfo> {
        let semester_id = self.semester_id.ok_or_else(|| {
            crate::error::MmsError::Other(
                "Semester ID is required. Use .in_semester(id) before calling .create()"
                    .to_string(),
            )
        })?;

        create_course(
            db,
            semester_id,
            self.short_name,
            self.name,
            self.ects,
            self.lecturer,
            self.lecturer_email,
            self.tutor,
            self.tutor_email,
            self.learning_platform_url,
            self.university,
            self.location,
            self.is_external,
            self.original_path,
            self.has_git_repo,
            self.git_remote_url,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let builder = CourseBuilder::new("cs101", "Introduction to Algorithms", 8);
        assert_eq!(builder.short_name, "cs101");
        assert_eq!(builder.name, "Introduction to Algorithms");
        assert_eq!(builder.ects, 8);
        assert_eq!(builder.semester_id, None);
        assert_eq!(builder.is_external, false);
    }

    #[test]
    fn test_builder_with_details() {
        let builder = CourseBuilder::new("math201", "Linear Algebra", 6)
            .in_semester(1)
            .with_lecturer("Prof. Wagner")
            .with_lecturer_email("wagner@tum.de")
            .with_tutor("Thomas Weber")
            .with_university("TUM")
            .with_git_repo("https://github.com/user/math201");

        assert_eq!(builder.semester_id, Some(1));
        assert_eq!(builder.lecturer, Some("Prof. Wagner".to_string()));
        assert_eq!(builder.lecturer_email, Some("wagner@tum.de".to_string()));
        assert_eq!(builder.tutor, Some("Thomas Weber".to_string()));
        assert_eq!(builder.university, Some("TUM".to_string()));
        assert_eq!(builder.has_git_repo, true);
        assert_eq!(
            builder.git_remote_url,
            Some("https://github.com/user/math201".to_string())
        );
    }

    #[test]
    fn test_builder_external_course() {
        let builder = CourseBuilder::new("external101", "External Course", 5)
            .in_semester(2)
            .as_external(Some("/old/path/to/course".to_string()));

        assert_eq!(builder.is_external, true);
        assert_eq!(
            builder.original_path,
            Some("/old/path/to/course".to_string())
        );
    }
}
