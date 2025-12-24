use crate::config::Config;
use crate::db::entities::semesters;
use crate::error::{MmsError, Result};
use crate::toml::{SemesterToml, SemesterType};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::path::PathBuf;

/// Information about a semester (returned after creation)
#[derive(Debug, Clone)]
pub struct SemesterInfo {
    /// Database ID
    pub id: i64,
    /// Semester type (Bachelor or Master)
    pub semester_type: SemesterType,
    /// Semester number
    pub number: i32,
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

impl From<semesters::Model> for SemesterInfo {
    fn from(model: semesters::Model) -> Self {
        let semester_type = SemesterType::from_str(&model.r#type).unwrap_or(SemesterType::Bachelor);

        Self {
            id: model.id,
            semester_type,
            number: model.number as i32,
            code: format!("{}{}", semester_type.prefix(), model.number),
            directory_path: PathBuf::from(model.directory_path),
            start_date: model.start_date,
            end_date: model.end_date,
            university: model.university,
            location: model.default_location,
            is_current: model.is_current,
            is_archived: model.is_archived,
        }
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
/// * `start_date` - Optional start date (ISO format: YYYY-MM-DD)
/// * `end_date` - Optional end date (ISO format: YYYY-MM-DD)
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
    number: i32,
    start_date: Option<String>,
    end_date: Option<String>,
    university: Option<String>,
    location: Option<String>,
    is_current: bool,
    is_archived: bool,
) -> Result<SemesterInfo> {
    // Get base path from config (validated on load)
    let base_path = &config.university_base_path;

    // Generate semester code (e.g., "b3")
    let code = format!("{}{}", semester_type.prefix(), number);

    // Create semester directory path
    let semester_dir = base_path.join(&code);

    // Create the directory
    std::fs::create_dir_all(&semester_dir).map_err(|e| {
        MmsError::Other(format!(
            "Failed to create semester directory {}: {}",
            semester_dir.display(),
            e
        ))
    })?;

    // Create and write .semester.toml
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

    toml.write_to_directory(&semester_dir)?;

    // Get location: use provided, or config default if available, otherwise None
    let final_location = location.or_else(|| {
        config.general.as_ref()
            .and_then(|g| g.default_location.clone())
    });

    // Create database entry
    let active_model = semesters::ActiveModel {
        id: ActiveValue::NotSet,
        r#type: ActiveValue::Set(semester_type.to_string()),
        number: ActiveValue::Set(number as i64),
        directory_path: ActiveValue::Set(semester_dir.to_string_lossy().to_string()),
        exists_on_disk: ActiveValue::Set(true),
        last_scanned_at: ActiveValue::Set(Some(Utc::now())),
        start_date: ActiveValue::Set(start_date.clone()),
        end_date: ActiveValue::Set(end_date.clone()),
        default_location: ActiveValue::Set(final_location.clone()),
        university: ActiveValue::Set(university.clone()),
        is_current: ActiveValue::Set(is_current),
        is_archived: ActiveValue::Set(is_archived),
        created_at: ActiveValue::Set(Utc::now()),
        updated_at: ActiveValue::Set(Utc::now()),
    };

    let model = active_model.insert(db).await?;

    Ok(SemesterInfo {
        id: model.id,
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

    // Update TOML file
    let semester_dir = PathBuf::from(&semester.directory_path);
    if semester_dir.exists() {
        let mut toml = SemesterToml::read_from_directory(&semester_dir).unwrap_or_else(|_| {
            // If TOML doesn't exist or can't be read, create new one
            let sem_type =
                SemesterType::from_str(&semester.r#type).unwrap_or(SemesterType::Bachelor);
            SemesterToml::new(sem_type, semester.number as i32)
        });

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
    Ok(updated.into())
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
                MmsError::Other(format!(
                    "Failed to delete semester directory {}: {}",
                    semester_dir.display(),
                    e
                ))
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

    Ok(semester.into())
}

/// Get a semester by its code (e.g., "b3", "m1")
pub async fn get_semester_by_code(db: &DatabaseConnection, code: &str) -> Result<SemesterInfo> {
    // Parse code into type and number
    let first_char = code
        .chars()
        .next()
        .ok_or_else(|| MmsError::Parse("Invalid semester code".to_string()))?;

    let semester_type = match first_char {
        'b' | 'B' => "bachelor",
        'm' | 'M' => "master",
        _ => {
            return Err(MmsError::Parse(format!(
                "Invalid semester type prefix: {}",
                first_char
            )));
        }
    };

    let number: i64 = code[1..]
        .parse()
        .map_err(|_| MmsError::Parse(format!("Invalid semester number in code: {}", code)))?;

    let semester = semesters::Entity::find()
        .filter(semesters::Column::Type.eq(semester_type))
        .filter(semesters::Column::Number.eq(number))
        .one(db)
        .await?
        .ok_or_else(|| MmsError::NotFound(format!("Semester {} not found", code)))?;

    Ok(semester.into())
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

    Ok(semesters.into_iter().map(SemesterInfo::from).collect())
}
