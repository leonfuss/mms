use crate::db::entities::{courses, semesters};
use crate::error::{MmsError, Result};
use crate::toml::CourseToml;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::path::PathBuf;

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
/// use mms_core::course::create_course;
/// use mms_core::config::Config;
/// use sea_orm::DatabaseConnection;
///
/// # async fn example(config: &Config, db: &DatabaseConnection) -> mms_core::error::Result<()> {
/// let course = create_course(
///     config,
///     db,
///     1, // semester_id
///     "cs101".to_string(),
///     "Introduction to Algorithms".to_string(),
///     8, // ECTS
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
pub async fn create_course(
    db: &DatabaseConnection,
    semester_id: i64,
    short_name: String,
    name: String,
    ects: i32,
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
    // Get semester from database
    let semester = semesters::Entity::find_by_id(semester_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::SemesterNotFound(semester_id))?;

    // Get semester directory
    let semester_dir = PathBuf::from(&semester.directory_path);

    // Create course directory path
    let course_dir = semester_dir.join(&short_name);

    // Create the directory (unless it's external and original_path is set)
    if !is_external || original_path.is_none() {
        std::fs::create_dir_all(&course_dir).map_err(|e| {
            MmsError::Other(format!(
                "Failed to create course directory {}: {}",
                course_dir.display(),
                e
            ))
        })?;
    }

    // Create and write .course.toml (skip if external with original_path)
    let toml_path = if !is_external || original_path.is_none() {
        let mut toml = CourseToml::new(short_name.clone(), name.clone(), ects);

        if let Some(l) = &lecturer {
            toml = toml.with_lecturer(l.clone());
        }
        if let Some(e) = &lecturer_email {
            toml = toml.with_lecturer_email(e.clone());
        }
        if let Some(t) = &tutor {
            toml = toml.with_tutor(t.clone());
        }
        if let Some(e) = &tutor_email {
            toml = toml.with_tutor_email(e.clone());
        }
        if let Some(url) = &learning_platform_url {
            toml = toml.with_learning_platform_url(url.clone());
        }
        if let Some(uni) = &university {
            toml = toml.with_university(uni.clone());
        }
        if let Some(loc) = &location {
            toml = toml.with_location(loc.clone());
        }
        if is_external {
            toml = toml.with_external(true);
            if let Some(orig) = &original_path {
                toml = toml.with_original_path(orig.clone());
            }
        }
        if has_git_repo && let Some(url) = &git_remote_url {
            toml = toml.with_git_repo(url.clone());
        }

        toml.write_to_directory(&course_dir)?;

        Some(course_dir.join(".course.toml"))
    } else {
        None
    };

    // Determine which directory path to use
    let final_directory_path = match (is_external, &original_path) {
        (true, Some(path)) => path.clone(),
        (_, _) => course_dir.to_string_lossy().to_string(),
    };

    // Create database entry
    let active_model = courses::ActiveModel {
        id: ActiveValue::NotSet,
        semester_id: ActiveValue::Set(semester_id),
        short_name: ActiveValue::Set(short_name.clone()),
        name: ActiveValue::Set(name.clone()),
        directory_path: ActiveValue::Set(final_directory_path.clone()),
        toml_path: ActiveValue::Set(toml_path.as_ref().map(|p| p.to_string_lossy().to_string())),
        exists_on_disk: ActiveValue::Set(!is_external || original_path.is_none()),
        toml_exists: ActiveValue::Set(toml_path.is_some()),
        last_scanned_at: ActiveValue::Set(Some(Utc::now())),
        ects: ActiveValue::Set(ects as i64),
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

    let model = active_model.insert(db).await?;

    Ok(CourseInfo {
        id: model.id,
        semester_id,
        short_name,
        name,
        directory_path: PathBuf::from(final_directory_path),
        toml_path,
        ects,
        lecturer,
        lecturer_email,
        tutor,
        tutor_email,
        learning_platform_url,
        university,
        location,
        is_external,
        original_path,
        has_git_repo,
        git_remote_url,
    })
}

/// Update an existing course
///
/// Updates both the database entry and the .course.toml file
/// Only non-None values are updated
pub async fn update_course(
    db: &DatabaseConnection,
    course_id: i64,
    name: Option<String>,
    ects: Option<i32>,
    lecturer: Option<String>,
    lecturer_email: Option<String>,
    tutor: Option<String>,
    tutor_email: Option<String>,
    learning_platform_url: Option<String>,
    university: Option<String>,
    location: Option<String>,
) -> Result<CourseInfo> {
    // Get existing course
    let course = courses::Entity::find_by_id(course_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::CourseNotFound(course_id))?;

    // Update TOML file if it exists
    let course_dir = PathBuf::from(&course.directory_path);
    if !course.is_external && course_dir.exists() {
        let mut toml = CourseToml::read_from_directory(&course_dir).unwrap_or_else(|_| {
            // If TOML doesn't exist, create new one
            CourseToml::new(
                course.short_name.clone(),
                course.name.clone(),
                course.ects as i32,
            )
        });

        // Apply updates
        if let Some(n) = &name {
            toml.name = n.clone();
        }
        if let Some(e) = ects {
            toml.ects = e;
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

        toml.write_to_directory(&course_dir)?;
    }

    // Update database
    let mut active_model: courses::ActiveModel = course.into();

    if let Some(n) = name {
        active_model.name = ActiveValue::Set(n);
    }
    if let Some(e) = ects {
        active_model.ects = ActiveValue::Set(e as i64);
    }
    if let Some(l) = lecturer {
        active_model.lecturer = ActiveValue::Set(Some(l));
    }
    if let Some(e) = lecturer_email {
        active_model.lecturer_email = ActiveValue::Set(Some(e));
    }
    if let Some(t) = tutor {
        active_model.tutor = ActiveValue::Set(Some(t));
    }
    if let Some(e) = tutor_email {
        active_model.tutor_email = ActiveValue::Set(Some(e));
    }
    if let Some(url) = learning_platform_url {
        active_model.learning_platform_url = ActiveValue::Set(Some(url));
    }
    if let Some(uni) = university {
        active_model.university = ActiveValue::Set(Some(uni));
    }
    if let Some(loc) = location {
        active_model.location = ActiveValue::Set(Some(loc));
    }

    active_model.updated_at = ActiveValue::Set(Utc::now());

    let updated = active_model.update(db).await?;
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
