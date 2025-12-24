use crate::config::Config;
use crate::db::entities::semesters;
use crate::error::{MmsError, Result};
use crate::toml::{SemesterToml, SemesterType};
use chrono::NaiveDate;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    TransactionTrait,
};
use std::path::PathBuf;

/// Validate German date format (DD.MM.YYYY)
///
/// Accepts flexible formats:
/// - 12.02.2004 (with leading zeros)
/// - 7.8.2003 (without leading zeros)
/// - 01.12.2024 (mixed)
fn validate_date_format(date: &str) -> Result<()> {
    parse_german_date(date)?;
    Ok(())
}

/// Parse German date format (DD.MM.YYYY) to NaiveDate
///
/// Uses chrono's padding-agnostic parsing, which accepts both:
/// - Padded: "07.08.2003"
/// - Non-padded: "7.8.2003"
fn parse_german_date(date: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date, "%d.%m.%Y").map_err(MmsError::ChronoParse)
}

/// Validate start_date < end_date
fn validate_date_range(start: &Option<String>, end: &Option<String>) -> Result<()> {
    match (start, end) {
        (Some(s), Some(e)) => {
            let start_date = parse_german_date(s)?;
            let end_date = parse_german_date(e)?;

            if start_date >= end_date {
                return Err(MmsError::InvalidDateRange {
                    start: s.clone(),
                    end: e.clone(),
                });
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Validate semester number is positive
fn validate_semester_number(number: i64) -> Result<()> {
    if number <= 0 {
        return Err(MmsError::InvalidSemesterNumber { number });
    }
    Ok(())
}

/// Information about a semester (returned after creation)
#[derive(Debug, Clone)]
pub struct SemesterInfo {
    /// Database ID
    pub id: i64,
    /// Semester type (Bachelor or Master)
    pub semester_type: SemesterType,
    /// Semester number
    pub number: i64,
    /// Semester code (e.g., "b3")
    pub code: String,
    /// Full path to semester directory
    pub directory_path: PathBuf,
    /// Start date
    pub start_date: Option<String>,
    /// End date
    pub end_date: Option<String>,
    /// University
    pub university: Option<String>,
    /// Location
    pub location: Option<String>,
    /// Whether this semester is current
    pub is_current: bool,
    /// Whether this semester is archived
    pub is_archived: bool,
}

impl TryFrom<semesters::Model> for SemesterInfo {
    type Error = MmsError;

    fn try_from(model: semesters::Model) -> Result<Self> {
        let semester_type = SemesterType::from_str(&model.r#type)
            .ok_or_else(|| MmsError::InvalidSemesterType(model.r#type.clone()))?;

        Ok(Self {
            id: model.id,
            semester_type,
            number: model.number,
            code: format!("{}{}", semester_type.prefix(), model.number),
            directory_path: PathBuf::from(model.directory_path),
            start_date: model.start_date,
            end_date: model.end_date,
            university: model.university,
            location: model.default_location,
            is_current: model.is_current,
            is_archived: model.is_archived,
        })
    }
}

/// Create a new semester with folder, TOML file, and database entry
///
/// This function performs the following steps:
/// 1. Creates the semester directory (e.g., ~/Studies/b3/)
/// 2. Writes the .semester.toml file
/// 3. Inserts the semester into the database
///
/// # Arguments
/// * `config` - Application configuration
/// * `db` - Database connection
/// * `semester_type` - Bachelor or Master
/// * `number` - Semester number
/// * `start_date` - Optional start date (German format: DD.MM.YYYY, e.g., "01.10.2024")
/// * `end_date` - Optional end date (German format: DD.MM.YYYY, e.g., "31.03.2025")
/// * `university` - Optional university (uses config default if None)
/// * `location` - Optional location (uses config default if None)
/// * `is_current` - Whether this is the current active semester
/// * `is_archived` - Whether this semester is archived
///
/// # Returns
/// Information about the created semester
///
/// # Example
/// ```no_run
/// use mms_core::semester::{create_semester, SemesterType};
/// use mms_core::config::Config;
/// use sea_orm::DatabaseConnection;
///
/// # async fn example(config: &Config, db: &DatabaseConnection) -> mms_core::error::Result<()> {
/// let semester = create_semester(
///     config,
///     db,
///     SemesterType::Bachelor,
///     3,
///     Some("2024-10-01".to_string()),
///     Some("2025-03-31".to_string()),
///     Some("TUM".to_string()),
///     None, // Use config default location
///     true, // Set as current semester
///     false, // Not archived
/// ).await?;
///
/// println!("Created semester {} at {:?}", semester.code, semester.directory_path);
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_arguments)]
pub async fn create_semester(
    config: &Config,
    db: &DatabaseConnection,
    semester_type: SemesterType,
    number: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    university: Option<String>,
    location: Option<String>,
    is_current: bool,
    is_archived: bool,
) -> Result<SemesterInfo> {
    // VALIDATION (outside transaction - fail fast)
    validate_semester_number(number)?;

    if let Some(ref date) = start_date {
        validate_date_format(date)?;
    }
    if let Some(ref date) = end_date {
        validate_date_format(date)?;
    }

    validate_date_range(&start_date, &end_date)?;

    // Setup paths
    let base_path = &config.university_base_path;
    let code = format!("{}{}", semester_type.prefix(), number);
    let semester_dir = base_path.join(&code);

    let final_location = location.as_ref().cloned().or_else(|| {
        config
            .general
            .as_ref()
            .and_then(|g| g.default_location.clone())
    });

    // Use transaction for atomicity
    let txn = db.begin().await?;

    // Insert database record
    let active_model = semesters::ActiveModel {
        id: ActiveValue::NotSet,
        r#type: ActiveValue::Set(semester_type.to_string()),
        number: ActiveValue::Set(number),
        directory_path: ActiveValue::Set(semester_dir.to_string_lossy().to_string()),
        exists_on_disk: ActiveValue::Set(false),
        last_scanned_at: ActiveValue::Set(None),
        start_date: ActiveValue::Set(start_date.clone()),
        end_date: ActiveValue::Set(end_date.clone()),
        default_location: ActiveValue::Set(final_location.clone()),
        university: ActiveValue::Set(university.clone()),
        is_current: ActiveValue::Set(is_current),
        is_archived: ActiveValue::Set(is_archived),
        created_at: ActiveValue::Set(Utc::now()),
        updated_at: ActiveValue::Set(Utc::now()),
    };

    let model = active_model.insert(&txn).await?;

    // Create directory
    if let Err(e) = std::fs::create_dir_all(&semester_dir) {
        // Rollback transaction on filesystem error
        txn.rollback().await?;
        return Err(MmsError::SemesterDirectoryCreation {
            path: semester_dir.clone(),
            source: e,
        });
    }

    // Create TOML
    let mut toml = SemesterToml::new(semester_type, number);
    if let Some(start) = &start_date {
        toml = toml.with_start_date(start.clone());
    }
    if let Some(end) = &end_date {
        toml = toml.with_end_date(end.clone());
    }
    if let Some(uni) = &university {
        toml = toml.with_university(uni.clone());
    }
    if let Some(loc) = &location {
        toml = toml.with_location(loc.clone());
    }
    toml = toml.with_current(is_current).with_archived(is_archived);

    if let Err(e) = toml.write_to_directory(&semester_dir) {
        // Rollback transaction and cleanup directory
        txn.rollback().await?;
        let _ = std::fs::remove_dir_all(&semester_dir);
        return Err(e);
    }

    // Update exists_on_disk flag
    let mut update_model: semesters::ActiveModel = model.clone().into();
    update_model.exists_on_disk = ActiveValue::Set(true);
    update_model.last_scanned_at = ActiveValue::Set(Some(Utc::now()));
    let final_model = update_model.update(&txn).await?;

    // Commit transaction
    txn.commit().await?;

    Ok(SemesterInfo {
        id: final_model.id,
        semester_type,
        number,
        code,
        directory_path: semester_dir,
        start_date,
        end_date,
        university,
        location: final_location,
        is_current,
        is_archived,
    })
}

/// Update an existing semester
///
/// Updates both the database entry and the .semester.toml file
#[allow(clippy::too_many_arguments)]
pub async fn update_semester(
    db: &DatabaseConnection,
    semester_id: i64,
    start_date: Option<String>,
    end_date: Option<String>,
    university: Option<String>,
    location: Option<String>,
    is_current: Option<bool>,
    is_archived: Option<bool>,
) -> Result<SemesterInfo> {
    // Get existing semester
    let semester = semesters::Entity::find_by_id(semester_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::SemesterNotFound(semester_id))?;

    // VALIDATION
    if let Some(ref date) = start_date {
        validate_date_format(date)?;
    }
    if let Some(ref date) = end_date {
        validate_date_format(date)?;
    }

    // Validate combined state (new + existing)
    let final_start = start_date.as_ref().or(semester.start_date.as_ref());
    let final_end = end_date.as_ref().or(semester.end_date.as_ref());
    validate_date_range(&final_start.cloned(), &final_end.cloned())?;

    // Update TOML file
    let semester_dir = PathBuf::from(&semester.directory_path);
    if semester_dir.exists() {
        let mut toml = match SemesterToml::read_from_directory(&semester_dir) {
            Ok(t) => t,
            Err(_) => {
                // If TOML doesn't exist or can't be read, create new one from DB state
                let sem_type = SemesterType::from_str(&semester.r#type)
                    .ok_or_else(|| MmsError::InvalidSemesterType(semester.r#type.clone()))?;
                SemesterToml::new(sem_type, semester.number)
            }
        };

        // Apply updates
        if let Some(start) = &start_date {
            toml.start_date = Some(start.clone());
        }
        if let Some(end) = &end_date {
            toml.end_date = Some(end.clone());
        }
        if let Some(uni) = &university {
            toml.university = Some(uni.clone());
        }
        if let Some(loc) = &location {
            toml.location = Some(loc.clone());
        }
        if let Some(current) = is_current {
            toml.is_current = current;
        }
        if let Some(archived) = is_archived {
            toml.is_archived = archived;
        }

        toml.write_to_directory(&semester_dir)?;
    }

    // Update database
    let mut active_model: semesters::ActiveModel = semester.into();

    if let Some(start) = start_date {
        active_model.start_date = ActiveValue::Set(Some(start));
    }
    if let Some(end) = end_date {
        active_model.end_date = ActiveValue::Set(Some(end));
    }
    if let Some(uni) = university {
        active_model.university = ActiveValue::Set(Some(uni));
    }
    if let Some(loc) = location {
        active_model.default_location = ActiveValue::Set(Some(loc));
    }
    if let Some(current) = is_current {
        active_model.is_current = ActiveValue::Set(current);
    }
    if let Some(archived) = is_archived {
        active_model.is_archived = ActiveValue::Set(archived);
    }

    active_model.updated_at = ActiveValue::Set(Utc::now());

    let updated = active_model.update(db).await?;
    updated.try_into()
}

/// Delete a semester (database entry and optionally the directory)
///
/// # Arguments
/// * `db` - Database connection
/// * `semester_id` - ID of the semester to delete
/// * `delete_directory` - Whether to also delete the semester directory from disk
pub async fn delete_semester(
    db: &DatabaseConnection,
    semester_id: i64,
    delete_directory: bool,
) -> Result<()> {
    // Get semester info
    let semester = semesters::Entity::find_by_id(semester_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::SemesterNotFound(semester_id))?;

    // Delete from database (this will cascade to courses, etc.)
    semesters::Entity::delete_by_id(semester_id)
        .exec(db)
        .await?;

    // Optionally delete directory
    if delete_directory {
        let semester_dir = PathBuf::from(&semester.directory_path);
        if semester_dir.exists() {
            std::fs::remove_dir_all(&semester_dir).map_err(|e| {
                MmsError::SemesterDirectoryDeletion {
                    path: semester_dir.clone(),
                    source: e,
                }
            })?;
        }
    }

    Ok(())
}

/// Get a semester by its database ID
pub async fn get_semester_by_id(db: &DatabaseConnection, semester_id: i64) -> Result<SemesterInfo> {
    let semester = semesters::Entity::find_by_id(semester_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::SemesterNotFound(semester_id))?;

    semester.try_into()
}

/// Get a semester by its code (e.g., "b3", "m1")
pub async fn get_semester_by_code(db: &DatabaseConnection, code: &str) -> Result<SemesterInfo> {
    // Parse code into type and number
    let first_char = code
        .chars()
        .next()
        .ok_or_else(|| MmsError::InvalidSemesterCode {
            code: code.to_string(),
        })?;

    let semester_type = match first_char {
        'b' | 'B' => "bachelor",
        'm' | 'M' => "master",
        _ => {
            return Err(MmsError::InvalidSemesterCode {
                code: code.to_string(),
            });
        }
    };

    let number: i64 = code[1..]
        .parse()
        .map_err(|_| MmsError::InvalidSemesterCode {
            code: code.to_string(),
        })?;

    let semester = semesters::Entity::find()
        .filter(semesters::Column::Type.eq(semester_type))
        .filter(semesters::Column::Number.eq(number))
        .one(db)
        .await?
        .ok_or_else(|| MmsError::SemesterCodeNotFound {
            code: code.to_string(),
        })?;

    semester.try_into()
}

/// List all semesters
pub async fn list_semesters(
    db: &DatabaseConnection,
    include_archived: bool,
) -> Result<Vec<SemesterInfo>> {
    let mut query = semesters::Entity::find();

    if !include_archived {
        query = query.filter(semesters::Column::IsArchived.eq(false));
    }

    let semesters: Vec<semesters::Model> = query.all(db).await?;

    semesters
        .into_iter()
        .map(|s| s.try_into())
        .collect::<Result<Vec<_>>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_date_format_valid() {
        // German format with leading zeros
        assert!(validate_date_format("15.01.2024").is_ok());
        assert!(validate_date_format("31.12.2025").is_ok());

        // German format without leading zeros
        assert!(validate_date_format("7.8.2003").is_ok());
        assert!(validate_date_format("1.1.2024").is_ok());

        // Mixed formats
        assert!(validate_date_format("07.8.2003").is_ok());
        assert!(validate_date_format("7.08.2003").is_ok());
    }

    #[test]
    fn test_validate_date_format_invalid() {
        assert!(validate_date_format("01.13.2024").is_err()); // Invalid month
        assert!(validate_date_format("32.01.2024").is_err()); // Invalid day
        assert!(validate_date_format("29.02.2023").is_err()); // Invalid leap year
        assert!(validate_date_format("01/01/2024").is_err()); // Wrong separator (slash)
        assert!(validate_date_format("2024-01-01").is_err()); // ISO format (wrong)
        assert!(validate_date_format("01-01-2024").is_err()); // Wrong separator (dash)
        assert!(validate_date_format("not-a-date").is_err());
        assert!(validate_date_format("").is_err()); // Empty string
        assert!(validate_date_format("1.2").is_err()); // Missing year
        assert!(validate_date_format("1.2.3.4").is_err()); // Too many parts
    }

    #[test]
    fn test_validate_date_range_valid() {
        let start = Some("01.01.2024".to_string());
        let end = Some("31.12.2024".to_string());
        assert!(validate_date_range(&start, &end).is_ok());

        // Test with non-padded dates
        let start2 = Some("1.1.2024".to_string());
        let end2 = Some("31.12.2024".to_string());
        assert!(validate_date_range(&start2, &end2).is_ok());

        // None values should pass
        assert!(validate_date_range(&None, &end).is_ok());
        assert!(validate_date_range(&start, &None).is_ok());
        assert!(validate_date_range(&None, &None).is_ok());
    }

    #[test]
    fn test_validate_date_range_invalid() {
        let start = Some("31.12.2024".to_string());
        let end = Some("01.01.2024".to_string());

        let result = validate_date_range(&start, &end);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidDateRange { .. }
        ));

        // Equal dates should also fail
        let same = Some("15.06.2024".to_string());
        assert!(validate_date_range(&same, &same).is_err());
    }

    #[test]
    fn test_validate_semester_number_valid() {
        assert!(validate_semester_number(1).is_ok());
        assert!(validate_semester_number(999).is_ok());
        assert!(validate_semester_number(i64::MAX).is_ok());
    }

    #[test]
    fn test_validate_semester_number_invalid() {
        assert!(validate_semester_number(0).is_err());
        assert!(validate_semester_number(-1).is_err());
        assert!(validate_semester_number(i64::MIN).is_err());

        let result = validate_semester_number(0);
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidSemesterNumber { number: 0 }
        ));
    }
}
