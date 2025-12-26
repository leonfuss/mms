use std::str::FromStr;

use crate::db::entities::{course_degree_mappings, courses, degree_areas, degrees};
use crate::degree::builder::AreaDefinition;
use crate::degree::types::{AreaEcts, DegreeEcts, DegreeType};
use crate::error::{MmsError, Result};
use crate::utils::date_validation::{validate_date_format, validate_date_range};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, TransactionTrait,
};

/// Information about a degree program (returned after creation)
#[derive(Debug, Clone)]
pub struct DegreeInfo {
    /// Database ID
    pub id: i64,
    /// Degree type (Bachelor, Master, PhD)
    pub degree_type: DegreeType,
    /// Degree name
    pub name: String,
    /// University
    pub university: String,
    /// Total ECTS required
    pub total_ects_required: i32,
    /// Start date
    pub start_date: Option<String>,
    /// Expected end date
    pub expected_end_date: Option<String>,
    /// Whether this degree is active
    pub is_active: bool,
    /// Associated degree areas
    pub areas: Vec<DegreeAreaInfo>,
}

/// Information about a degree area (category)
#[derive(Debug, Clone)]
pub struct DegreeAreaInfo {
    /// Database ID
    pub id: i64,
    /// Degree ID
    pub degree_id: i64,
    /// Category name
    pub category_name: String,
    /// Required ECTS for this area
    pub required_ects: i32,
    /// Whether this area counts toward GPA
    pub counts_towards_gpa: bool,
    /// Display order
    pub display_order: i32,
}

/// Degree progress information
#[derive(Debug, Clone)]
pub struct DegreeProgressInfo {
    /// Degree ID
    pub degree_id: i64,
    /// Degree name
    pub degree_name: String,
    /// Total ECTS required
    pub total_ects_required: i32,
    /// Total ECTS earned (across all areas)
    pub total_ects_earned: i32,
    /// Progress per area
    pub area_progress: Vec<AreaProgress>,
    /// Overall GPA (only from areas that count)
    pub overall_gpa: Option<f64>,
}

/// Progress for a specific degree area
#[derive(Debug, Clone)]
pub struct AreaProgress {
    /// Area name
    pub category_name: String,
    /// Required ECTS
    pub required_ects: i32,
    /// Earned ECTS
    pub earned_ects: i32,
    /// Counts toward GPA
    pub counts_towards_gpa: bool,
    /// GPA for this area (if applicable)
    pub area_gpa: Option<f64>,
}

impl TryFrom<degrees::Model> for DegreeInfo {
    type Error = MmsError;

    fn try_from(model: degrees::Model) -> Result<Self> {
        let degree_type = DegreeType::from_str(&model.r#type)
            .map_err(|_| MmsError::InvalidDegreeType(model.r#type.clone()))?;

        Ok(Self {
            id: model.id,
            degree_type,
            name: model.name,
            university: model.university,
            total_ects_required: model.total_ects_required as i32,
            start_date: model.start_date,
            expected_end_date: model.expected_end_date,
            is_active: model.is_active,
            areas: Vec::new(), // Populated separately
        })
    }
}

impl From<degree_areas::Model> for DegreeAreaInfo {
    fn from(model: degree_areas::Model) -> Self {
        Self {
            id: model.id,
            degree_id: model.degree_id,
            category_name: model.category_name,
            required_ects: model.required_ects as i32,
            counts_towards_gpa: model.counts_towards_gpa,
            display_order: model.display_order as i32,
        }
    }
}

/// Create a new degree with associated areas
///
/// # Example
/// ```no_run
/// use mms_core::degree::{create_degree, DegreeType};
/// use mms_core::degree::builder::AreaDefinition;
/// use sea_orm::DatabaseConnection;
///
/// # async fn example(db: &DatabaseConnection) -> mms_core::error::Result<()> {
/// let areas = vec![
///     AreaDefinition {
///         category_name: "Core CS".to_string(),
///         required_ects: 60,
///         counts_towards_gpa: true,
///         display_order: 0,
///     },
/// ];
///
/// let degree = create_degree(
///     db,
///     DegreeType::Bachelor,
///     "Computer Science".to_string(),
///     "TUM".to_string(),
///     180,
///     Some("2020-10-01".to_string()),
///     Some("2023-09-30".to_string()),
///     true,
///     areas,
/// ).await?;
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_arguments)]
pub async fn create_degree(
    db: &DatabaseConnection,
    degree_type: DegreeType,
    name: String,
    university: String,
    total_ects_required: i32,
    start_date: Option<String>,
    expected_end_date: Option<String>,
    is_active: bool,
    areas: Vec<AreaDefinition>,
) -> Result<DegreeInfo> {
    // Validation (fail fast)
    DegreeEcts::new(total_ects_required, degree_type)?;

    if let Some(ref date) = start_date {
        validate_date_format(date)?;
    }
    if let Some(ref date) = expected_end_date {
        validate_date_format(date)?;
    }
    validate_date_range(&start_date, &expected_end_date)?;

    // Validate all areas
    for area in &areas {
        AreaEcts::new(area.required_ects)?;
    }

    // BEGIN TRANSACTION (all-or-nothing)
    let txn = db.begin().await?;

    // Create degree entry (on transaction)
    let degree_model = degrees::ActiveModel {
        id: ActiveValue::NotSet,
        r#type: ActiveValue::Set(degree_type.to_string()),
        name: ActiveValue::Set(name.clone()),
        university: ActiveValue::Set(university.clone()),
        total_ects_required: ActiveValue::Set(total_ects_required as i64),
        is_active: ActiveValue::Set(is_active),
        start_date: ActiveValue::Set(start_date.clone()),
        expected_end_date: ActiveValue::Set(expected_end_date.clone()),
        created_at: ActiveValue::Set(Utc::now()),
        updated_at: ActiveValue::Set(Utc::now()),
    };

    let degree = degree_model.insert(&txn).await?;

    // Create areas (on same transaction)
    let mut area_infos = Vec::new();
    for area in areas {
        let area_model = degree_areas::ActiveModel {
            id: ActiveValue::NotSet,
            degree_id: ActiveValue::Set(degree.id),
            category_name: ActiveValue::Set(area.category_name.clone()),
            required_ects: ActiveValue::Set(area.required_ects as i64),
            counts_towards_gpa: ActiveValue::Set(area.counts_towards_gpa),
            display_order: ActiveValue::Set(area.display_order as i64),
            created_at: ActiveValue::Set(Utc::now()),
        };

        let area_record = area_model.insert(&txn).await?;
        area_infos.push(DegreeAreaInfo::from(area_record));
    }

    // COMMIT TRANSACTION (if we got here, everything succeeded)
    txn.commit().await?;

    Ok(DegreeInfo {
        id: degree.id,
        degree_type,
        name,
        university,
        total_ects_required,
        start_date,
        expected_end_date,
        is_active,
        areas: area_infos,
    })
}

/// Update an existing degree
pub async fn update_degree(
    db: &DatabaseConnection,
    degree_id: i64,
    name: Option<String>,
    total_ects_required: Option<i32>,
    start_date: Option<String>,
    expected_end_date: Option<String>,
    is_active: Option<bool>,
) -> Result<DegreeInfo> {
    // Get existing degree
    let degree = degrees::Entity::find_by_id(degree_id)
        .one(db)
        .await?
        .ok_or(MmsError::DegreeNotFound(degree_id))?;

    // Validation (fail fast)
    let degree_type = DegreeType::from_str(&degree.r#type)
        .map_err(|_| MmsError::InvalidDegreeType(degree.r#type.clone()))?;

    if let Some(ects) = total_ects_required {
        DegreeEcts::new(ects, degree_type)?;
    }
    if let Some(ref date) = start_date {
        validate_date_format(date)?;
    }
    if let Some(ref date) = expected_end_date {
        validate_date_format(date)?;
    }

    // Validate date range with updated values
    let new_start = start_date.as_ref().or(degree.start_date.as_ref());
    let new_end = expected_end_date
        .as_ref()
        .or(degree.expected_end_date.as_ref());
    validate_date_range(&new_start.cloned(), &new_end.cloned())?;

    // Update degree
    let mut active_model: degrees::ActiveModel = degree.into();

    if let Some(n) = name {
        active_model.name = ActiveValue::Set(n);
    }
    if let Some(ects) = total_ects_required {
        active_model.total_ects_required = ActiveValue::Set(ects as i64);
    }
    if let Some(date) = start_date {
        active_model.start_date = ActiveValue::Set(Some(date));
    }
    if let Some(date) = expected_end_date {
        active_model.expected_end_date = ActiveValue::Set(Some(date));
    }
    if let Some(active) = is_active {
        active_model.is_active = ActiveValue::Set(active);
    }

    active_model.updated_at = ActiveValue::Set(Utc::now());

    let updated = active_model.update(db).await?;

    // Get areas
    let areas = get_degree_areas(db, degree_id).await?;

    let mut info = DegreeInfo::try_from(updated)?;
    info.areas = areas;

    Ok(info)
}

/// Delete a degree (and cascade to areas and mappings)
pub async fn delete_degree(db: &DatabaseConnection, degree_id: i64) -> Result<()> {
    let result = degrees::Entity::delete_by_id(degree_id).exec(db).await?;

    if result.rows_affected == 0 {
        return Err(MmsError::DegreeNotFound(degree_id));
    }

    Ok(())
}

/// Get a degree by its database ID
pub async fn get_degree_by_id(db: &DatabaseConnection, degree_id: i64) -> Result<DegreeInfo> {
    let degree = degrees::Entity::find_by_id(degree_id)
        .one(db)
        .await?
        .ok_or(MmsError::DegreeNotFound(degree_id))?;

    let areas = get_degree_areas(db, degree_id).await?;

    let mut info = DegreeInfo::try_from(degree)?;
    info.areas = areas;

    Ok(info)
}

/// List all degrees
pub async fn list_degrees(
    db: &DatabaseConnection,
    include_inactive: bool,
) -> Result<Vec<DegreeInfo>> {
    let mut query = degrees::Entity::find();

    if !include_inactive {
        query = query.filter(degrees::Column::IsActive.eq(true));
    }

    let degrees_list = query
        .order_by_asc(degrees::Column::Type)
        .order_by_asc(degrees::Column::Name)
        .all(db)
        .await?;

    let mut result = Vec::new();
    for degree in degrees_list {
        let areas = get_degree_areas(db, degree.id).await?;
        let mut info = DegreeInfo::try_from(degree)?;
        info.areas = areas;
        result.push(info);
    }

    Ok(result)
}

/// Get all areas for a degree
async fn get_degree_areas(db: &DatabaseConnection, degree_id: i64) -> Result<Vec<DegreeAreaInfo>> {
    let areas = degree_areas::Entity::find()
        .filter(degree_areas::Column::DegreeId.eq(degree_id))
        .order_by_asc(degree_areas::Column::DisplayOrder)
        .all(db)
        .await?;

    Ok(areas.into_iter().map(DegreeAreaInfo::from).collect())
}

/// Add a new area to a degree
pub async fn add_degree_area(
    db: &DatabaseConnection,
    degree_id: i64,
    category_name: String,
    required_ects: i32,
    counts_towards_gpa: bool,
) -> Result<DegreeAreaInfo> {
    // Check if degree exists
    degrees::Entity::find_by_id(degree_id)
        .one(db)
        .await?
        .ok_or(MmsError::DegreeNotFound(degree_id))?;

    // Get next display order
    let existing_areas = degree_areas::Entity::find()
        .filter(degree_areas::Column::DegreeId.eq(degree_id))
        .order_by_desc(degree_areas::Column::DisplayOrder)
        .all(db)
        .await?;

    let display_order = existing_areas
        .first()
        .map(|a| a.display_order + 1)
        .unwrap_or(0);

    let area_model = degree_areas::ActiveModel {
        id: ActiveValue::NotSet,
        degree_id: ActiveValue::Set(degree_id),
        category_name: ActiveValue::Set(category_name),
        required_ects: ActiveValue::Set(required_ects as i64),
        counts_towards_gpa: ActiveValue::Set(counts_towards_gpa),
        display_order: ActiveValue::Set(display_order),
        created_at: ActiveValue::Set(Utc::now()),
    };

    let area = area_model.insert(db).await?;
    Ok(DegreeAreaInfo::from(area))
}

/// Update an existing degree area
pub async fn update_degree_area(
    db: &DatabaseConnection,
    area_id: i64,
    category_name: Option<String>,
    required_ects: Option<i32>,
    counts_towards_gpa: Option<bool>,
) -> Result<DegreeAreaInfo> {
    let area = degree_areas::Entity::find_by_id(area_id)
        .one(db)
        .await?
        .ok_or(MmsError::DegreeAreaNotFound(area_id))?;

    let mut active_model: degree_areas::ActiveModel = area.into();

    if let Some(name) = category_name {
        active_model.category_name = ActiveValue::Set(name);
    }
    if let Some(ects) = required_ects {
        active_model.required_ects = ActiveValue::Set(ects as i64);
    }
    if let Some(counts) = counts_towards_gpa {
        active_model.counts_towards_gpa = ActiveValue::Set(counts);
    }

    let updated = active_model.update(db).await?;
    Ok(DegreeAreaInfo::from(updated))
}

/// Delete a degree area
pub async fn delete_degree_area(db: &DatabaseConnection, area_id: i64) -> Result<()> {
    let result = degree_areas::Entity::delete_by_id(area_id).exec(db).await?;

    if result.rows_affected == 0 {
        return Err(MmsError::NotFound(format!(
            "Degree area with ID {} not found",
            area_id
        )));
    }

    Ok(())
}

/// Map a course to a degree area (mark it as counting toward that area)
pub async fn map_course_to_area(
    db: &DatabaseConnection,
    course_id: i64,
    degree_id: i64,
    area_id: i64,
    ects_override: Option<i32>,
) -> Result<()> {
    // Verify course, degree, and area exist
    courses::Entity::find_by_id(course_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::CourseNotFound(course_id))?;

    degrees::Entity::find_by_id(degree_id)
        .one(db)
        .await?
        .ok_or(MmsError::DegreeNotFound(degree_id))?;

    degree_areas::Entity::find_by_id(area_id)
        .one(db)
        .await?
        .ok_or(MmsError::DegreeAreaNotFound(area_id))?;

    // Create mapping
    let mapping = course_degree_mappings::ActiveModel {
        id: ActiveValue::NotSet,
        course_id: ActiveValue::Set(course_id),
        degree_id: ActiveValue::Set(degree_id),
        area_id: ActiveValue::Set(area_id),
        ects_override: ActiveValue::Set(ects_override.map(|e| e as i64)),
        created_at: ActiveValue::Set(Utc::now()),
    };

    mapping.insert(db).await?;
    Ok(())
}

/// Remove a course from a degree area
pub async fn unmap_course_from_area(
    db: &DatabaseConnection,
    course_id: i64,
    degree_id: i64,
    area_id: i64,
) -> Result<()> {
    let result = course_degree_mappings::Entity::delete_many()
        .filter(course_degree_mappings::Column::CourseId.eq(course_id))
        .filter(course_degree_mappings::Column::DegreeId.eq(degree_id))
        .filter(course_degree_mappings::Column::AreaId.eq(area_id))
        .exec(db)
        .await?;

    if result.rows_affected == 0 {
        return Err(MmsError::NotFound(format!(
            "Course mapping not found for course {}, degree {}, area {}",
            course_id, degree_id, area_id
        )));
    }

    Ok(())
}

/// Get degree progress (uses database views for GPA calculation)
pub async fn get_degree_progress(
    db: &DatabaseConnection,
    degree_id: i64,
) -> Result<DegreeProgressInfo> {
    use sea_orm::FromQueryResult;

    // Get degree info
    let degree = get_degree_by_id(db, degree_id).await?;

    // Query the v_degree_progress view
    #[derive(Debug, FromQueryResult)]
    struct ViewProgress {
        category_name: String,
        required_ects: i64,
        earned_ects: i64,
        area_gpa: Option<f64>,
        counts_towards_gpa: bool,
    }

    let progress_data: Vec<ViewProgress> =
        ViewProgress::find_by_statement(sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"SELECT category_name, required_ects, earned_ects, area_gpa, counts_towards_gpa
               FROM v_degree_progress
               WHERE degree_id = ?"#,
            vec![degree_id.into()],
        ))
        .all(db)
        .await?;

    let mut area_progress = Vec::new();
    let mut total_earned = 0;
    let mut weighted_gpa_sum = 0.0;
    let mut gpa_ects = 0;

    for area in progress_data {
        total_earned += area.earned_ects as i32;

        if area.counts_towards_gpa
            && let Some(gpa) = area.area_gpa
        {
            weighted_gpa_sum += gpa * area.earned_ects as f64;
            gpa_ects += area.earned_ects as i32;
        }

        area_progress.push(AreaProgress {
            category_name: area.category_name,
            required_ects: area.required_ects as i32,
            earned_ects: area.earned_ects as i32,
            counts_towards_gpa: area.counts_towards_gpa,
            area_gpa: area.area_gpa,
        });
    }

    let overall_gpa = if gpa_ects > 0 {
        Some(weighted_gpa_sum / gpa_ects as f64)
    } else {
        None
    };

    Ok(DegreeProgressInfo {
        degree_id,
        degree_name: degree.name,
        total_ects_required: degree.total_ects_required,
        total_ects_earned: total_earned,
        area_progress,
        overall_gpa,
    })
}

/// Get courses that have not been mapped to any degree area
pub async fn get_unmapped_courses(db: &DatabaseConnection) -> Result<Vec<i64>> {
    use sea_orm::FromQueryResult;

    #[derive(Debug, FromQueryResult)]
    struct UnmappedCourse {
        id: i64,
    }

    let unmapped: Vec<UnmappedCourse> =
        UnmappedCourse::find_by_statement(sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"SELECT c.id
               FROM courses c
               LEFT JOIN course_degree_mappings cdm ON c.id = cdm.course_id
               WHERE c.is_archived = 0
                 AND c.is_dropped = 0
                 AND cdm.id IS NULL"#,
            vec![],
        ))
        .all(db)
        .await?;

    Ok(unmapped.into_iter().map(|c| c.id).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    #[test]
    fn test_date_validation_import() {
        // Test that shared date validation is accessible
        assert!(validate_date_format("01.10.2024").is_ok());
        assert!(validate_date_format("99.99.2024").is_err());
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    /// Set up a test environment with in-memory database
    async fn setup_test_db() -> Result<DatabaseConnection> {
        let db = Database::connect("sqlite::memory:").await?;

        // Run migrations
        crate::db::migrations::run_migrations(&db).await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_integration_full_crud_lifecycle() {
        let db = setup_test_db().await.unwrap();

        // CREATE
        let areas = vec![
            AreaDefinition {
                category_name: "Core CS".to_string(),
                required_ects: 60,
                counts_towards_gpa: true,
                display_order: 0,
            },
            AreaDefinition {
                category_name: "Electives".to_string(),
                required_ects: 30,
                counts_towards_gpa: true,
                display_order: 1,
            },
        ];

        let degree = create_degree(
            &db,
            DegreeType::Bachelor,
            "Computer Science".to_string(),
            "TUM".to_string(),
            180,
            Some("01.10.2024".to_string()),
            Some("30.09.2028".to_string()),
            true,
            areas,
        )
        .await
        .unwrap();

        assert_eq!(degree.name, "Computer Science");
        assert_eq!(degree.university, "TUM");
        assert_eq!(degree.total_ects_required, 180);
        assert_eq!(degree.degree_type, DegreeType::Bachelor);
        assert_eq!(degree.areas.len(), 2);
        assert_eq!(degree.areas[0].category_name, "Core CS");
        assert_eq!(degree.areas[0].required_ects, 60);
        assert_eq!(degree.areas[1].category_name, "Electives");
        assert_eq!(degree.areas[1].required_ects, 30);

        // READ
        let retrieved = get_degree_by_id(&db, degree.id).await.unwrap();
        assert_eq!(retrieved.id, degree.id);
        assert_eq!(retrieved.name, "Computer Science");
        assert_eq!(retrieved.areas.len(), 2);

        // LIST
        let all_degrees = list_degrees(&db, true).await.unwrap();
        assert_eq!(all_degrees.len(), 1);

        // UPDATE
        let updated = update_degree(
            &db,
            degree.id,
            Some("CS".to_string()),
            Some(190),
            None,
            None,
            Some(false),
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "CS");
        assert_eq!(updated.total_ects_required, 190);
        assert!(!updated.is_active);

        // DELETE
        delete_degree(&db, degree.id).await.unwrap();

        let result = get_degree_by_id(&db, degree.id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MmsError::DegreeNotFound(_)));
    }

    #[tokio::test]
    async fn test_invalid_bachelor_ects_rejected() {
        let db = setup_test_db().await.unwrap();

        let areas = vec![];

        // Too low (Bachelor needs 90-240)
        let result = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS".to_string(),
            "TUM".to_string(),
            50, // Invalid!
            None,
            None,
            true,
            areas.clone(),
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidDegreeEcts { .. }
        ));

        // Too high
        let result = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS".to_string(),
            "TUM".to_string(),
            300, // Invalid!
            None,
            None,
            true,
            areas,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidDegreeEcts { .. }
        ));
    }

    #[tokio::test]
    async fn test_invalid_master_ects_rejected() {
        let db = setup_test_db().await.unwrap();

        let areas = vec![];

        // Too low (Master needs 60-120)
        let result = create_degree(
            &db,
            DegreeType::Master,
            "CS".to_string(),
            "TUM".to_string(),
            30, // Invalid!
            None,
            None,
            true,
            areas.clone(),
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidDegreeEcts { .. }
        ));

        // Too high
        let result = create_degree(
            &db,
            DegreeType::Master,
            "CS".to_string(),
            "TUM".to_string(),
            150, // Invalid!
            None,
            None,
            true,
            areas,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidDegreeEcts { .. }
        ));
    }

    #[tokio::test]
    async fn test_phd_requires_zero_ects() {
        let db = setup_test_db().await.unwrap();

        let areas = vec![];

        // PhD must have 0 ECTS
        let result = create_degree(
            &db,
            DegreeType::PhD,
            "Computer Science".to_string(),
            "TUM".to_string(),
            60, // Invalid - PhD requires 0!
            None,
            None,
            true,
            areas.clone(),
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidDegreeEcts { .. }
        ));

        // Valid PhD with 0 ECTS
        let result = create_degree(
            &db,
            DegreeType::PhD,
            "Computer Science".to_string(),
            "TUM".to_string(),
            0, // Valid!
            None,
            None,
            true,
            areas,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_rollback_on_invalid_area() {
        let db = setup_test_db().await.unwrap();

        let areas = vec![
            AreaDefinition {
                category_name: "Core".to_string(),
                required_ects: 60,
                counts_towards_gpa: true,
                display_order: 0,
            },
            AreaDefinition {
                category_name: "Invalid".to_string(),
                required_ects: -10, // Invalid! Area ECTS must be positive
                counts_towards_gpa: true,
                display_order: 1,
            },
        ];

        let result = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS".to_string(),
            "TUM".to_string(),
            180,
            None,
            None,
            true,
            areas,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MmsError::InvalidAreaEcts(_)));

        // Verify no degree was created (transaction rolled back)
        let degrees = list_degrees(&db, true).await.unwrap();
        assert_eq!(degrees.len(), 0);
    }

    #[tokio::test]
    async fn test_invalid_date_format_rejected() {
        let db = setup_test_db().await.unwrap();

        let areas = vec![];

        // Invalid start date format
        let result = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS".to_string(),
            "TUM".to_string(),
            180,
            Some("2024-10-01".to_string()), // ISO format - wrong!
            None,
            true,
            areas.clone(),
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MmsError::ChronoParse(_)));

        // Invalid end date format
        let result = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS".to_string(),
            "TUM".to_string(),
            180,
            None,
            Some("99.99.2024".to_string()), // Invalid date
            true,
            areas,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MmsError::ChronoParse(_)));
    }

    #[tokio::test]
    async fn test_invalid_date_range_rejected() {
        let db = setup_test_db().await.unwrap();

        let areas = vec![];

        // End date before start date
        let result = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS".to_string(),
            "TUM".to_string(),
            180,
            Some("30.09.2028".to_string()),
            Some("01.10.2024".to_string()), // End before start!
            true,
            areas.clone(),
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidDateRange { .. }
        ));

        // Equal dates (start == end is invalid)
        let result = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS".to_string(),
            "TUM".to_string(),
            180,
            Some("01.10.2024".to_string()),
            Some("01.10.2024".to_string()), // Same date!
            true,
            areas,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidDateRange { .. }
        ));
    }

    #[tokio::test]
    async fn test_valid_german_date_formats_accepted() {
        let db = setup_test_db().await.unwrap();

        let areas = vec![];

        // Without leading zeros
        let result = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS1".to_string(),
            "TUM".to_string(),
            180,
            Some("1.10.2024".to_string()),
            Some("30.9.2028".to_string()),
            true,
            areas.clone(),
        )
        .await;

        assert!(result.is_ok());

        // With leading zeros
        let result = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS2".to_string(),
            "TUM".to_string(),
            180,
            Some("01.10.2024".to_string()),
            Some("30.09.2028".to_string()),
            true,
            areas,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_with_invalid_ects_rejected() {
        let db = setup_test_db().await.unwrap();

        // Create valid degree
        let degree = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS".to_string(),
            "TUM".to_string(),
            180,
            None,
            None,
            true,
            vec![],
        )
        .await
        .unwrap();

        // Try to update with invalid ECTS
        let result = update_degree(
            &db,
            degree.id,
            None,
            Some(50), // Invalid for Bachelor!
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidDegreeEcts { .. }
        ));

        // Verify degree wasn't updated
        let unchanged = get_degree_by_id(&db, degree.id).await.unwrap();
        assert_eq!(unchanged.total_ects_required, 180);
    }

    #[tokio::test]
    async fn test_update_with_invalid_date_range_rejected() {
        let db = setup_test_db().await.unwrap();

        // Create degree with valid dates
        let degree = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS".to_string(),
            "TUM".to_string(),
            180,
            Some("01.10.2024".to_string()),
            Some("30.09.2028".to_string()),
            true,
            vec![],
        )
        .await
        .unwrap();

        // Try to update end date to be before start date
        let result = update_degree(
            &db,
            degree.id,
            None,
            None,
            None,
            Some("01.09.2024".to_string()), // Before existing start date!
            None,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmsError::InvalidDateRange { .. }
        ));
    }

    #[tokio::test]
    async fn test_area_management() {
        let db = setup_test_db().await.unwrap();

        // Create degree with initial area
        let degree = create_degree(
            &db,
            DegreeType::Bachelor,
            "CS".to_string(),
            "TUM".to_string(),
            180,
            None,
            None,
            true,
            vec![AreaDefinition {
                category_name: "Core".to_string(),
                required_ects: 90,
                counts_towards_gpa: true,
                display_order: 0,
            }],
        )
        .await
        .unwrap();

        assert_eq!(degree.areas.len(), 1);

        // Add new area
        let new_area = add_degree_area(&db, degree.id, "Electives".to_string(), 60, true)
            .await
            .unwrap();

        assert_eq!(new_area.category_name, "Electives");
        assert_eq!(new_area.required_ects, 60);

        // Verify degree now has 2 areas
        let updated_degree = get_degree_by_id(&db, degree.id).await.unwrap();
        assert_eq!(updated_degree.areas.len(), 2);

        // Update area
        let area_id = updated_degree.areas[0].id;
        let modified_area =
            update_degree_area(&db, area_id, Some("Core CS".to_string()), Some(100), None)
                .await
                .unwrap();

        assert_eq!(modified_area.category_name, "Core CS");
        assert_eq!(modified_area.required_ects, 100);

        // Delete area
        delete_degree_area(&db, area_id).await.unwrap();

        let final_degree = get_degree_by_id(&db, degree.id).await.unwrap();
        assert_eq!(final_degree.areas.len(), 1);
        assert_eq!(final_degree.areas[0].category_name, "Electives");
    }
}
