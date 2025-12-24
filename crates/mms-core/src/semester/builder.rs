use crate::config::Config;
use crate::error::Result;
use crate::semester::operations::{SemesterInfo, create_semester};
use crate::toml::SemesterType;
use sea_orm::DatabaseConnection;

/// Builder for creating a new semester
///
/// # Example
/// ```no_run
/// use mms_core::semester::{SemesterBuilder, SemesterType};
/// use mms_core::config::Config;
/// use sea_orm::DatabaseConnection;
///
/// # async fn example(config: &Config, db: &DatabaseConnection) -> mms_core::error::Result<()> {
/// let semester = SemesterBuilder::new(SemesterType::Bachelor, 3)
///     .with_start_date("2024-10-01")
///     .with_end_date("2025-03-31")
///     .with_university("TUM")
///     .with_current(true)
///     .create(config, db)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SemesterBuilder {
    semester_type: SemesterType,
    number: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    university: Option<String>,
    location: Option<String>,
    is_current: bool,
    is_archived: bool,
}

impl SemesterBuilder {
    /// Create a new semester builder with required fields
    ///
    /// # Arguments
    /// * `semester_type` - Bachelor or Master
    /// * `number` - Semester number (e.g., 1, 2, 3...)
    pub fn new(semester_type: SemesterType, number: i64) -> Self {
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

    /// Set the start date (German format: DD.MM.YYYY, e.g., "15.01.2024" or "7.8.2003")
    pub fn with_start_date<S: Into<String>>(mut self, date: S) -> Self {
        self.start_date = Some(date.into());
        self
    }

    /// Set the end date (German format: DD.MM.YYYY, e.g., "31.12.2024" or "7.8.2003")
    pub fn with_end_date<S: Into<String>>(mut self, date: S) -> Self {
        self.end_date = Some(date.into());
        self
    }

    /// Set the university (overrides config default)
    pub fn with_university<S: Into<String>>(mut self, university: S) -> Self {
        self.university = Some(university.into());
        self
    }

    /// Set the location (overrides config default)
    pub fn with_location<S: Into<String>>(mut self, location: S) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Mark this semester as the current active semester
    pub fn with_current(mut self, is_current: bool) -> Self {
        self.is_current = is_current;
        self
    }

    /// Mark this semester as archived
    pub fn with_archived(mut self, is_archived: bool) -> Self {
        self.is_archived = is_archived;
        self
    }

    /// Create the semester (folder + TOML + database entry)
    ///
    /// This method will:
    /// 1. Create the semester directory (e.g., ~/Studies/b3/)
    /// 2. Write the .semester.toml file
    /// 3. Create the database entry
    ///
    /// Returns the created semester info
    pub async fn create(self, config: &Config, db: &DatabaseConnection) -> Result<SemesterInfo> {
        create_semester(
            config,
            db,
            self.semester_type,
            self.number,
            self.start_date,
            self.end_date,
            self.university,
            self.location,
            self.is_current,
            self.is_archived,
        )
        .await
    }

    /// Get the semester code (e.g., "b3", "m1")
    pub fn code(&self) -> String {
        format!("{}{}", self.semester_type.prefix(), self.number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let builder = SemesterBuilder::new(SemesterType::Bachelor, 3);
        assert_eq!(builder.semester_type, SemesterType::Bachelor);
        assert_eq!(builder.number, 3);
        assert_eq!(builder.code(), "b3");
        assert_eq!(builder.start_date, None);
        assert!(!builder.is_current);
    }

    #[test]
    fn test_builder_with_dates() {
        let builder = SemesterBuilder::new(SemesterType::Master, 2)
            .with_start_date("2024-10-01")
            .with_end_date("2025-03-31")
            .with_university("TUM")
            .with_current(true);

        assert_eq!(builder.start_date, Some("2024-10-01".to_string()));
        assert_eq!(builder.end_date, Some("2025-03-31".to_string()));
        assert_eq!(builder.university, Some("TUM".to_string()));
        assert!(builder.is_current);
    }

    #[test]
    fn test_builder_code_generation() {
        assert_eq!(SemesterBuilder::new(SemesterType::Bachelor, 1).code(), "b1");
        assert_eq!(SemesterBuilder::new(SemesterType::Master, 5).code(), "m5");
    }
}
