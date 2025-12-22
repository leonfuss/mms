use crate::db::entities::{course_degree_mappings, courses, degree_areas, degrees};
use crate::degree::builder::AreaDefinition;
use crate::degree::types::DegreeType;
use crate::error::{MmsError, Result};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use std::collections::HashMap;

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

impl From<degrees::Model> for DegreeInfo {
    fn from(model: degrees::Model) -> Self {
        let degree_type = DegreeType::from_str(&model.r#type).unwrap_or(DegreeType::Bachelor);

        Self {
            id: model.id,
            degree_type,
            name: model.name,
            university: model.university,
            total_ects_required: model.total_ects_required as i32,
            start_date: model.start_date,
            expected_end_date: model.expected_end_date,
            is_active: model.is_active,
            areas: Vec::new(), // Populated separately
        }
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
    // Create degree entry
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

    let degree = degree_model.insert(db).await?;

    // Create areas
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

        let area_record = area_model.insert(db).await?;
        area_infos.push(DegreeAreaInfo::from(area_record));
    }

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
        .ok_or_else(|| MmsError::NotFound(format!("Degree with ID {} not found", degree_id)))?;

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

    let mut info = DegreeInfo::from(updated);
    info.areas = areas;

    Ok(info)
}

/// Delete a degree (and cascade to areas and mappings)
pub async fn delete_degree(db: &DatabaseConnection, degree_id: i64) -> Result<()> {
    let result = degrees::Entity::delete_by_id(degree_id).exec(db).await?;

    if result.rows_affected == 0 {
        return Err(MmsError::NotFound(format!(
            "Degree with ID {} not found",
            degree_id
        )));
    }

    Ok(())
}

/// Get a degree by its database ID
pub async fn get_degree_by_id(db: &DatabaseConnection, degree_id: i64) -> Result<DegreeInfo> {
    let degree = degrees::Entity::find_by_id(degree_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::NotFound(format!("Degree with ID {} not found", degree_id)))?;

    let areas = get_degree_areas(db, degree_id).await?;

    let mut info = DegreeInfo::from(degree);
    info.areas = areas;

    Ok(info)
}

/// List all degrees
pub async fn list_degrees(db: &DatabaseConnection, include_inactive: bool) -> Result<Vec<DegreeInfo>> {
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
        let mut info = DegreeInfo::from(degree);
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
        .ok_or_else(|| MmsError::NotFound(format!("Degree with ID {} not found", degree_id)))?;

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
        .ok_or_else(|| MmsError::NotFound(format!("Degree area with ID {} not found", area_id)))?;

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
    let result = degree_areas::Entity::delete_by_id(area_id)
        .exec(db)
        .await?;

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
        .ok_or_else(|| MmsError::NotFound(format!("Degree with ID {} not found", degree_id)))?;

    degree_areas::Entity::find_by_id(area_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::NotFound(format!("Degree area with ID {} not found", area_id)))?;

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

    let progress_data: Vec<ViewProgress> = ViewProgress::find_by_statement(
        sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"SELECT category_name, required_ects, earned_ects, area_gpa, counts_towards_gpa
               FROM v_degree_progress
               WHERE degree_id = ?"#,
            vec![degree_id.into()],
        ),
    )
    .all(db)
    .await?;

    let mut area_progress = Vec::new();
    let mut total_earned = 0;
    let mut weighted_gpa_sum = 0.0;
    let mut gpa_ects = 0;

    for area in progress_data {
        total_earned += area.earned_ects as i32;

        if area.counts_towards_gpa {
            if let Some(gpa) = area.area_gpa {
                weighted_gpa_sum += gpa * area.earned_ects as f64;
                gpa_ects += area.earned_ects as i32;
            }
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

    let unmapped: Vec<UnmappedCourse> = UnmappedCourse::find_by_statement(
        sea_orm::Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"SELECT c.id
               FROM courses c
               LEFT JOIN course_degree_mappings cdm ON c.id = cdm.course_id
               WHERE c.is_archived = 0
                 AND c.is_dropped = 0
                 AND cdm.id IS NULL"#,
            vec![],
        ),
    )
    .all(db)
    .await?;

    Ok(unmapped.into_iter().map(|c| c.id).collect())
}
