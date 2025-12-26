use crate::course::types::{CourseCode, Ects};
use crate::db::entities::{courses, semesters};
use crate::error::{MmsError, Result};
use crate::toml::CourseToml;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    TransactionTrait,
};
use std::path::PathBuf;

// ============================================================================
// Validation Functions
// ============================================================================

/// Validates that a semester exists in the database
pub(crate) async fn validate_semester_exists(
    db: &DatabaseConnection,
    semester_id: i64,
) -> Result<()> {
    semesters::Entity::find_by_id(semester_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::SemesterNotFound(semester_id))?;
    Ok(())
}

// ============================================================================
// Data Transfer Objects
// ============================================================================

/// Information about a course (returned after creation)
#[derive(Debug, Clone)]
pub struct CourseInfo {
    /// Database ID
    pub id: i64,
    /// Semester ID
    pub semester_id: i64,
    /// Short name/code (e.g., "cs101")
    pub short_name: String,
    /// Full course name
    pub name: String,
    /// Full path to course directory
    pub directory_path: PathBuf,
    /// Path to .course.toml file
    pub toml_path: Option<PathBuf>,
    /// ECTS credits
    pub ects: i32,
    /// Lecturer name
    pub lecturer: Option<String>,
    /// Lecturer email
    pub lecturer_email: Option<String>,
    /// Tutor name
    pub tutor: Option<String>,
    /// Tutor email
    pub tutor_email: Option<String>,
    /// Learning platform URL
    pub learning_platform_url: Option<String>,
    /// University
    pub university: Option<String>,
    /// Location
    pub location: Option<String>,
    /// Is external course
    pub is_external: bool,
    /// Original path if external
    pub original_path: Option<String>,
    /// Has git repository
    pub has_git_repo: bool,
    /// Git remote URL
    pub git_remote_url: Option<String>,
}

impl From<courses::Model> for CourseInfo {
    fn from(model: courses::Model) -> Self {
        Self {
            id: model.id,
            semester_id: model.semester_id,
            short_name: model.short_name,
            name: model.name,
            directory_path: PathBuf::from(&model.directory_path),
            toml_path: model.toml_path.map(PathBuf::from),
            ects: model.ects as i32,
            lecturer: model.lecturer,
            lecturer_email: model.lecturer_email,
            tutor: model.tutor,
            tutor_email: model.tutor_email,
            learning_platform_url: model.learning_platform_url,
            university: model.university,
            location: model.location,
            is_external: model.is_external,
            original_path: model.original_path,
            has_git_repo: model.has_git_repo,
            git_remote_url: model.git_remote_url,
        }
    }
}

/// Create a new course with folder, TOML file, and database entry
///
/// This function performs the following steps:
/// 1. Gets the semester directory path from the database
/// 2. Creates the course directory (e.g., ~/Studies/b3/cs101/)
/// 3. Writes the .course.toml file
/// 4. Inserts the course into the database
///
/// # Example
/// ```no_run
/// use mms_core::course::{create_course, CourseCode, Ects};
/// use mms_core::config::Config;
/// use sea_orm::DatabaseConnection;
///
/// # async fn example(config: &Config, db: &DatabaseConnection) -> mms_core::error::Result<()> {
/// let course = create_course(
///     db,
///     1, // semester_id
///     CourseCode::new("cs101".to_string())?,
///     "Introduction to Algorithms".to_string(),
///     Ects::new(8)?,
///     Some("Prof. Dr. Schmidt".to_string()),
///     Some("schmidt@tum.de".to_string()),
///     Some("Anna Müller".to_string()),
///     Some("anna@tum.de".to_string()),
///     Some("https://moodle.tum.de/course/123".to_string()),
///     Some("TUM".to_string()),
///     None, // Use semester default location
///     false, // Not external
///     None, // No original path
///     true, // Has git repo
///     Some("https://github.com/user/cs101".to_string()),
/// ).await?;
///
/// println!("Created course {} at {:?}", course.short_name, course.directory_path);
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_arguments)]
pub async fn create_course(
    db: &DatabaseConnection,
    semester_id: i64,
    short_name: CourseCode,
    name: String,
    ects: Ects,
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
) -> Result<CourseInfo> {
    // Validate semester exists (types guarantee ECTS and course code validity)
    validate_semester_exists(db, semester_id).await?;

    // Get semester from database
    let semester = semesters::Entity::find_by_id(semester_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::SemesterNotFound(semester_id))?;

    // Get semester directory
    let semester_dir = PathBuf::from(&semester.directory_path);

    // Create course directory path
    let course_dir = semester_dir.join(short_name.as_str());

    // Determine which directory path to use for external courses
    let final_directory_path = match (is_external, &original_path) {
        (true, Some(path)) => path.clone(),
        (_, _) => course_dir.to_string_lossy().to_string(),
    };

    // Prepare TOML configuration (we'll write it after DB insert in transaction)
    let mut toml = CourseToml::new(short_name.as_str().to_string(), name.clone(), *ects);
    if let Some(l) = &lecturer {
        toml = toml.with_lecturer(l);
    }
    if let Some(e) = &lecturer_email {
        toml = toml.with_lecturer_email(e);
    }
    if let Some(t) = &tutor {
        toml = toml.with_tutor(t);
    }
    if let Some(e) = &tutor_email {
        toml = toml.with_tutor_email(e);
    }
    if let Some(url) = &learning_platform_url {
        toml = toml.with_learning_platform_url(url);
    }
    if let Some(uni) = &university {
        toml = toml.with_university(uni);
    }
    if let Some(loc) = &location {
        toml = toml.with_location(loc);
    }
    if is_external {
        toml = toml.with_external(true);
        if let Some(orig) = &original_path {
            toml = toml.with_original_path(orig);
        }
    }
    if has_git_repo && let Some(url) = &git_remote_url {
        toml = toml.with_git_repo(url);
    }

    // BEGIN TRANSACTION - ensures atomicity of DB + filesystem operations
    let txn = db.begin().await?;

    // Insert database entry with exists_on_disk=false (filesystem not created yet)
    let active_model = courses::ActiveModel {
        id: ActiveValue::NotSet,
        semester_id: ActiveValue::Set(semester_id),
        short_name: ActiveValue::Set(short_name.as_str().to_string()),
        name: ActiveValue::Set(name.clone()),
        directory_path: ActiveValue::Set(final_directory_path.clone()),
        toml_path: ActiveValue::NotSet, // Will be set after TOML is written
        exists_on_disk: ActiveValue::Set(false), // Not created yet!
        toml_exists: ActiveValue::Set(false), // Not created yet!
        last_scanned_at: ActiveValue::Set(None), // Will be set after creation
        ects: ActiveValue::Set(*ects as i64),
        lecturer: ActiveValue::Set(lecturer.clone()),
        lecturer_email: ActiveValue::Set(lecturer_email.clone()),
        tutor: ActiveValue::Set(tutor.clone()),
        tutor_email: ActiveValue::Set(tutor_email.clone()),
        learning_platform_url: ActiveValue::Set(learning_platform_url.clone()),
        university: ActiveValue::Set(university.clone()),
        location: ActiveValue::Set(location.clone()),
        is_external: ActiveValue::Set(is_external),
        original_path: ActiveValue::Set(original_path.clone()),
        is_archived: ActiveValue::Set(false),
        is_dropped: ActiveValue::Set(false),
        dropped_at: ActiveValue::Set(None),
        has_git_repo: ActiveValue::Set(has_git_repo),
        git_remote_url: ActiveValue::Set(git_remote_url.clone()),
        created_at: ActiveValue::Set(Utc::now()),
        updated_at: ActiveValue::Set(Utc::now()),
    };

    let model = active_model.insert(&txn).await?;

    // Create filesystem resources (with explicit rollback on failure)
    let (toml_path, should_create_fs) = if !is_external || original_path.is_none() {
        // Create the directory
        if let Err(e) = std::fs::create_dir_all(&course_dir) {
            txn.rollback().await?;
            return Err(MmsError::CourseDirectoryCreation {
                path: course_dir.clone(),
                source: e,
            });
        }

        // Write .course.toml
        if let Err(e) = toml.write_to_directory(&course_dir) {
            txn.rollback().await?;
            return Err(MmsError::Other(format!(
                "Failed to write .course.toml: {}",
                e
            )));
        }

        (Some(course_dir.join(".course.toml")), true)
    } else {
        (None, false)
    };

    // Update database flags to reflect filesystem state
    let mut active_update: courses::ActiveModel = model.clone().into();
    active_update.toml_path =
        ActiveValue::Set(toml_path.as_ref().map(|p| p.to_string_lossy().to_string()));
    active_update.exists_on_disk = ActiveValue::Set(should_create_fs);
    active_update.toml_exists = ActiveValue::Set(toml_path.is_some());
    active_update.last_scanned_at = ActiveValue::Set(Some(Utc::now()));

    let final_model = active_update.update(&txn).await?;

    // COMMIT TRANSACTION - All operations succeeded!
    txn.commit().await?;

    // Return CourseInfo from the final database model
    Ok(CourseInfo {
        id: final_model.id,
        semester_id: final_model.semester_id,
        short_name: final_model.short_name,
        name: final_model.name,
        directory_path: PathBuf::from(&final_model.directory_path),
        toml_path,
        ects: final_model.ects as i32,
        lecturer: final_model.lecturer,
        lecturer_email: final_model.lecturer_email,
        tutor: final_model.tutor,
        tutor_email: final_model.tutor_email,
        learning_platform_url: final_model.learning_platform_url,
        university: final_model.university,
        location: final_model.location,
        is_external: final_model.is_external,
        original_path: final_model.original_path,
        has_git_repo: final_model.has_git_repo,
        git_remote_url: final_model.git_remote_url,
    })
}

/// Update an existing course
///
/// Updates both the database entry and the .course.toml file atomically.
/// Only non-None values are updated.
///
/// # Arguments
/// * `short_name` - New course code (will rename directory if changed)
/// * `force_recreate_toml` - If true, recreate TOML from DB if corrupted; if false, return error
///
/// # Errors
/// Returns `CorruptedCourseToml` if TOML is corrupted and `force_recreate_toml` is false
#[allow(clippy::too_many_arguments)]
pub async fn update_course(
    db: &DatabaseConnection,
    course_id: i64,
    short_name: Option<CourseCode>,
    name: Option<String>,
    ects: Option<Ects>,
    lecturer: Option<String>,
    lecturer_email: Option<String>,
    tutor: Option<String>,
    tutor_email: Option<String>,
    learning_platform_url: Option<String>,
    university: Option<String>,
    location: Option<String>,
    force_recreate_toml: bool,
) -> Result<CourseInfo> {
    // Types guarantee ECTS and course code validity

    // BEGIN TRANSACTION - ensures atomicity of DB + filesystem operations
    let txn = db.begin().await?;

    // Get existing course (on transaction)
    let course = courses::Entity::find_by_id(course_id)
        .one(&txn)
        .await?
        .ok_or_else(|| MmsError::CourseNotFound(course_id))?;

    let old_dir = PathBuf::from(&course.directory_path);
    let old_short_name = course.short_name.clone();

    // Handle short_name change (rename directory atomically)
    let (new_dir, directory_renamed) = if let Some(ref new_short_name) = short_name {
        if new_short_name.as_str() != old_short_name {
            // Calculate new directory path
            let parent = old_dir.parent().ok_or_else(|| {
                MmsError::Other(format!(
                    "Course directory has no parent: {}",
                    old_dir.display()
                ))
            })?;
            let new_dir = parent.join(new_short_name.as_str());

            // Rename directory (with rollback on failure)
            if !course.is_external && old_dir.exists() {
                if let Err(e) = std::fs::rename(&old_dir, &new_dir) {
                    txn.rollback().await?;
                    return Err(MmsError::Other(format!(
                        "Failed to rename course directory from {} to {}: {}",
                        old_dir.display(),
                        new_dir.display(),
                        e
                    )));
                }
                (new_dir, true)
            } else {
                (new_dir, false)
            }
        } else {
            (old_dir.clone(), false)
        }
    } else {
        (old_dir.clone(), false)
    };

    // Build database updates
    let mut active_model: courses::ActiveModel = course.clone().into();

    // Keep a reference to short_name for later TOML update
    let short_name_str = short_name.as_ref().map(|sn| sn.as_str().to_string());

    if let Some(sn) = short_name {
        active_model.short_name = ActiveValue::Set(sn.into_string());
        active_model.directory_path = ActiveValue::Set(new_dir.to_string_lossy().to_string());
    }
    if let Some(n) = &name {
        active_model.name = ActiveValue::Set(n.clone());
    }
    if let Some(e) = ects {
        active_model.ects = ActiveValue::Set(*e as i64);
    }
    if let Some(l) = &lecturer {
        active_model.lecturer = ActiveValue::Set(Some(l.clone()));
    }
    if let Some(e) = &lecturer_email {
        active_model.lecturer_email = ActiveValue::Set(Some(e.clone()));
    }
    if let Some(t) = &tutor {
        active_model.tutor = ActiveValue::Set(Some(t.clone()));
    }
    if let Some(e) = &tutor_email {
        active_model.tutor_email = ActiveValue::Set(Some(e.clone()));
    }
    if let Some(url) = &learning_platform_url {
        active_model.learning_platform_url = ActiveValue::Set(Some(url.clone()));
    }
    if let Some(uni) = &university {
        active_model.university = ActiveValue::Set(Some(uni.clone()));
    }
    if let Some(loc) = &location {
        active_model.location = ActiveValue::Set(Some(loc.clone()));
    }

    active_model.updated_at = ActiveValue::Set(Utc::now());

    // Update database (on transaction)
    let updated = active_model.update(&txn).await?;

    // Update TOML file (with explicit error handling and rollback)
    if !updated.is_external && new_dir.exists() {
        let toml_path = new_dir.join(".course.toml");

        // Read TOML (error by default, recreate if forced)
        let mut toml = match CourseToml::read_from_directory(&new_dir) {
            Ok(t) => t,
            Err(e) => {
                if force_recreate_toml {
                    eprintln!(
                        "Warning: Recreating corrupted .course.toml from database state: {}",
                        e
                    );
                    CourseToml::new(
                        updated.short_name.clone(),
                        updated.name.clone(),
                        updated.ects as i32,
                    )
                } else {
                    txn.rollback().await?;
                    // If directory was renamed, rename it back
                    if directory_renamed {
                        let _ = std::fs::rename(&new_dir, &old_dir);
                    }
                    return Err(MmsError::CorruptedCourseToml {
                        path: toml_path,
                        reason: e.to_string(),
                    });
                }
            }
        };

        // Apply TOML updates
        if let Some(ref sn) = short_name_str {
            toml.short_name = sn.clone();
        }
        if let Some(n) = &name {
            toml.name = n.clone();
        }
        if let Some(e) = ects {
            toml.ects = *e;
        }
        if let Some(l) = &lecturer {
            toml.lecturer = Some(l.clone());
        }
        if let Some(e) = &lecturer_email {
            toml.lecturer_email = Some(e.clone());
        }
        if let Some(t) = &tutor {
            toml.tutor = Some(t.clone());
        }
        if let Some(e) = &tutor_email {
            toml.tutor_email = Some(e.clone());
        }
        if let Some(url) = &learning_platform_url {
            toml.learning_platform_url = Some(url.clone());
        }
        if let Some(uni) = &university {
            toml.university = Some(uni.clone());
        }
        if let Some(loc) = &location {
            toml.location = Some(loc.clone());
        }

        // Write TOML (with rollback on failure)
        if let Err(e) = toml.write_to_directory(&new_dir) {
            txn.rollback().await?;
            // If directory was renamed, rename it back
            if directory_renamed {
                let _ = std::fs::rename(&new_dir, &old_dir);
            }
            return Err(MmsError::Other(format!(
                "Failed to write .course.toml: {}",
                e
            )));
        }
    }

    // COMMIT TRANSACTION - All operations succeeded!
    txn.commit().await?;

    Ok(updated.into())
}

/// Delete a course (database entry and optionally the directory)
///
/// # Arguments
/// * `db` - Database connection
/// * `course_id` - ID of the course to delete
/// * `delete_directory` - Whether to also delete the course directory from disk
pub async fn delete_course(
    db: &DatabaseConnection,
    course_id: i64,
    delete_directory: bool,
) -> Result<()> {
    // Get course info
    let course = courses::Entity::find_by_id(course_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::CourseNotFound(course_id))?;

    // Delete from database (this will cascade to related entries)
    courses::Entity::delete_by_id(course_id).exec(db).await?;

    // Optionally delete directory
    if delete_directory && !course.is_external {
        let course_dir = PathBuf::from(&course.directory_path);
        if course_dir.exists() {
            std::fs::remove_dir_all(&course_dir).map_err(|e| {
                MmsError::Other(format!(
                    "Failed to delete course directory {}: {}",
                    course_dir.display(),
                    e
                ))
            })?;
        }
    }

    Ok(())
}

/// Get a course by its database ID
pub async fn get_course_by_id(db: &DatabaseConnection, course_id: i64) -> Result<CourseInfo> {
    let course = courses::Entity::find_by_id(course_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::CourseNotFound(course_id))?;

    Ok(course.into())
}

/// Get a course by its short name within a semester
pub async fn get_course_by_short_name(
    db: &DatabaseConnection,
    semester_id: i64,
    short_name: &str,
) -> Result<CourseInfo> {
    let course = courses::Entity::find()
        .filter(courses::Column::SemesterId.eq(semester_id))
        .filter(courses::Column::ShortName.eq(short_name))
        .one(db)
        .await?
        .ok_or_else(|| {
            MmsError::NotFound(format!(
                "Course {} not found in semester {}",
                short_name, semester_id
            ))
        })?;

    Ok(course.into())
}

/// List all courses
pub async fn list_courses(
    db: &DatabaseConnection,
    semester_id: Option<i64>,
    include_archived: bool,
    include_dropped: bool,
) -> Result<Vec<CourseInfo>> {
    let mut query = courses::Entity::find();

    if let Some(sid) = semester_id {
        query = query.filter(courses::Column::SemesterId.eq(sid));
    }

    if !include_archived {
        query = query.filter(courses::Column::IsArchived.eq(false));
    }

    if !include_dropped {
        query = query.filter(courses::Column::IsDropped.eq(false));
    }

    let courses: Vec<courses::Model> = query.all(db).await?;

    Ok(courses.into_iter().map(CourseInfo::from).collect())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Integration Tests
    // ------------------------------------------------------------------------

    use crate::course::types::{CourseCode, Ects};
    use crate::db::entities::{prelude::*, semesters};
    use sea_orm::Database;
    use tempfile::TempDir;

    /// Set up a test environment with in-memory database and temp directory
    async fn setup_test_env() -> Result<(DatabaseConnection, TempDir)> {
        let db = Database::connect("sqlite::memory:").await?;

        // Run migrations
        crate::db::migrations::run_migrations(&db).await?;

        let temp_dir = TempDir::new()?;

        Ok((db, temp_dir))
    }

    /// Create a test semester for use in course tests
    async fn create_test_semester(db: &DatabaseConnection, temp_dir: &TempDir) -> Result<i64> {
        use sea_orm::ActiveValue;

        let semester_dir = temp_dir.path().join("b3");
        std::fs::create_dir_all(&semester_dir)?;

        let semester = semesters::ActiveModel {
            id: ActiveValue::NotSet,
            r#type: ActiveValue::Set("Bachelor".to_string()),
            number: ActiveValue::Set(3),
            directory_path: ActiveValue::Set(semester_dir.to_string_lossy().to_string()),
            exists_on_disk: ActiveValue::Set(true),
            last_scanned_at: ActiveValue::Set(None),
            start_date: ActiveValue::Set(Some("2024-10-01".to_string())),
            end_date: ActiveValue::Set(Some("2025-03-31".to_string())),
            default_location: ActiveValue::Set(Some("Munich".to_string())),
            university: ActiveValue::Set(Some("TUM".to_string())),
            is_current: ActiveValue::Set(true),
            is_archived: ActiveValue::Set(false),
            created_at: ActiveValue::Set(chrono::Utc::now()),
            updated_at: ActiveValue::Set(chrono::Utc::now()),
        };

        let model = semester.insert(db).await?;
        Ok(model.id)
    }

    #[tokio::test]
    async fn test_integration_full_crud_lifecycle() {
        let (db, temp_dir) = setup_test_env().await.unwrap();
        let semester_id = create_test_semester(&db, &temp_dir).await.unwrap();

        // CREATE
        let course = create_course(
            &db,
            semester_id,
            CourseCode::new("cs101".to_string()).unwrap(),
            "Intro to CS".to_string(),
            Ects::new(6).unwrap(),
            Some("Dr. Smith".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            false,
            None,
            false,
            None,
        )
        .await
        .unwrap();

        assert_eq!(course.short_name, "cs101");
        assert_eq!(course.name, "Intro to CS");
        assert_eq!(course.ects, 6);
        assert_eq!(course.lecturer, Some("Dr. Smith".to_string()));

        // Verify directory exists
        assert!(std::path::PathBuf::from(&course.directory_path).exists());

        // READ
        let retrieved = get_course_by_id(&db, course.id).await.unwrap();
        assert_eq!(retrieved.short_name, "cs101");

        let by_short_name = get_course_by_short_name(&db, semester_id, "cs101")
            .await
            .unwrap();
        assert_eq!(by_short_name.id, course.id);

        // LIST
        let courses = list_courses(&db, Some(semester_id), false, false)
            .await
            .unwrap();
        assert_eq!(courses.len(), 1);

        // UPDATE
        let updated = update_course(
            &db,
            course.id,
            None,
            Some("Introduction to Computer Science".to_string()),
            Some(Ects::new(9).unwrap()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "Introduction to Computer Science");
        assert_eq!(updated.ects, 9);

        // Verify TOML was updated
        let toml_path = std::path::PathBuf::from(&updated.directory_path).join(".course.toml");
        assert!(toml_path.exists());
        let toml = CourseToml::read(&toml_path).unwrap();
        assert_eq!(toml.name, "Introduction to Computer Science");
        assert_eq!(toml.ects, 9);

        // DELETE
        delete_course(&db, course.id, true).await.unwrap();

        // Verify deleted from database
        let result = get_course_by_id(&db, course.id).await;
        assert!(result.is_err());

        // Verify directory removed
        assert!(!std::path::PathBuf::from(&course.directory_path).exists());
    }

    #[tokio::test]
    async fn test_integration_external_course_handling() {
        let (db, temp_dir) = setup_test_env().await.unwrap();
        let semester_id = create_test_semester(&db, &temp_dir).await.unwrap();

        let course = create_course(
            &db,
            semester_id,
            CourseCode::new("external-cs".to_string()).unwrap(),
            "External Course".to_string(),
            Ects::new(6).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            true, // is_external = true
            Some("/external/path".to_string()),
            false,
            None,
        )
        .await
        .unwrap();

        assert!(course.is_external);
        assert_eq!(course.original_path, Some("/external/path".to_string()));

        // Verify directory NOT created for external course
        let course_dir = temp_dir.path().join("b3").join("external-cs");
        assert!(!course_dir.exists());

        // Verify database entry exists
        let db_course = get_course_by_id(&db, course.id).await.unwrap();
        assert_eq!(db_course.short_name, "external-cs");
        assert!(db_course.is_external);
    }

    #[tokio::test]
    async fn test_integration_invalid_ects_rejected() {
        let (db, temp_dir) = setup_test_env().await.unwrap();
        let semester_id = create_test_semester(&db, &temp_dir).await.unwrap();

        // Invalid ECTS should fail at type construction
        let result = Ects::new(0);
        assert!(result.is_err());

        let result = Ects::new(31);
        assert!(result.is_err());

        // Valid ECTS should work
        let course = create_course(
            &db,
            semester_id,
            CourseCode::new("cs101".to_string()).unwrap(),
            "Test Course".to_string(),
            Ects::new(6).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
            None,
            false,
            None,
        )
        .await
        .unwrap();

        assert_eq!(course.ects, 6);
    }

    #[tokio::test]
    async fn test_integration_invalid_course_code_rejected() {
        // Invalid codes should fail at type construction
        assert!(CourseCode::new("CS 101".to_string()).is_err()); // space
        assert!(CourseCode::new("foo/bar".to_string()).is_err()); // slash
        assert!(CourseCode::new(".hidden".to_string()).is_err()); // leading dot
        assert!(CourseCode::new("".to_string()).is_err()); // empty

        // Valid codes should work
        assert!(CourseCode::new("cs101".to_string()).is_ok());
        assert!(CourseCode::new("math-201".to_string()).is_ok());
    }

    #[tokio::test]
    async fn test_integration_nonexistent_semester_rejected() {
        let (db, _temp_dir) = setup_test_env().await.unwrap();

        let result = create_course(
            &db,
            99999, // nonexistent semester
            CourseCode::new("cs101".to_string()).unwrap(),
            "Test Course".to_string(),
            Ects::new(6).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
            None,
            false,
            None,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::SemesterNotFound(99999)
        ));
    }

    #[tokio::test]
    async fn test_integration_db_fs_sync_after_create() {
        let (db, temp_dir) = setup_test_env().await.unwrap();
        let semester_id = create_test_semester(&db, &temp_dir).await.unwrap();

        let course = create_course(
            &db,
            semester_id,
            CourseCode::new("cs101".to_string()).unwrap(),
            "Test Course".to_string(),
            Ects::new(6).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
            None,
            false,
            None,
        )
        .await
        .unwrap();

        // Verify DB flags match filesystem state
        let db_course = Courses::find_by_id(course.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert!(db_course.exists_on_disk);
        assert!(db_course.toml_exists);

        // Verify actual filesystem
        let course_dir = std::path::PathBuf::from(&db_course.directory_path);
        assert!(course_dir.exists());
        assert!(course_dir.join(".course.toml").exists());

        // Verify TOML contents
        let toml = CourseToml::read(&course_dir.join(".course.toml")).unwrap();
        assert_eq!(toml.short_name, "cs101");
        assert_eq!(toml.name, "Test Course");
        assert_eq!(toml.ects, 6);
    }

    #[tokio::test]
    async fn test_integration_update_with_short_name_change() {
        let (db, temp_dir) = setup_test_env().await.unwrap();
        let semester_id = create_test_semester(&db, &temp_dir).await.unwrap();

        // Create course
        let course = create_course(
            &db,
            semester_id,
            CourseCode::new("cs101".to_string()).unwrap(),
            "Test Course".to_string(),
            Ects::new(6).unwrap(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
            None,
            false,
            None,
        )
        .await
        .unwrap();

        let old_dir = std::path::PathBuf::from(&course.directory_path);
        assert!(old_dir.exists());

        // Update short_name (should rename directory)
        let updated = update_course(
            &db,
            course.id,
            Some(CourseCode::new("cs102".to_string()).unwrap()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
        )
        .await
        .unwrap();

        assert_eq!(updated.short_name, "cs102");

        // Verify old directory no longer exists
        assert!(!old_dir.exists());

        // Verify new directory exists
        let new_dir = std::path::PathBuf::from(&updated.directory_path);
        assert!(new_dir.exists());
        assert!(new_dir.join(".course.toml").exists());

        // Verify TOML has updated short_name
        let toml = CourseToml::read(&new_dir.join(".course.toml")).unwrap();
        assert_eq!(toml.short_name, "cs102");
    }
}
