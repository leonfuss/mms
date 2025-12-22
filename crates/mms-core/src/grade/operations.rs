use super::builder::ComponentDefinition;
use super::types::GradingScheme;
use crate::db::entities::{grade_components, grades, prelude::*};
use crate::error::{MmsError, Result};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

/// Complete grade information including components
#[derive(Debug, Clone)]
pub struct GradeInfo {
    pub id: i64,
    pub course_id: i64,
    pub grade: f64,
    pub grading_scheme: GradingScheme,
    pub original_grade: Option<f64>,
    pub original_scheme: Option<GradingScheme>,
    pub is_final: bool,
    pub passed: bool,
    pub attempt_number: i64,
    pub exam_date: Option<String>,
    pub recorded_at: String,
    pub components: Vec<ComponentInfo>,
}

/// Grade component information
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub id: i64,
    pub component_name: String,
    pub weight: f64,
    pub points_earned: Option<f64>,
    pub points_total: Option<f64>,
    pub grade: Option<f64>,
    pub is_bonus: bool,
    pub bonus_points: Option<f64>,
    pub is_completed: bool,
}

/// Record a new grade in the database
///
/// This function creates a grade entry and all its components atomically.
#[allow(clippy::too_many_arguments)]
pub async fn record_grade(
    db: &DatabaseConnection,
    course_id: i64,
    grade: f64,
    grading_scheme: GradingScheme,
    original_grade: Option<f64>,
    original_scheme: Option<GradingScheme>,
    is_final: bool,
    attempt_number: i64,
    exam_date: Option<String>,
    components: Vec<ComponentDefinition>,
) -> Result<GradeInfo> {
    // Validate grade for scheme
    if !grading_scheme.is_valid_grade(grade) {
        return Err(MmsError::Other(format!(
            "Grade {} is not valid for scheme {}",
            grade, grading_scheme
        )));
    }

    // Determine if passed
    let passed = grading_scheme.is_passing(grade);

    // Create grade entry
    let grade_model = grades::ActiveModel {
        course_id: Set(course_id),
        grade: Set(grade),
        grading_scheme: Set(grading_scheme.to_string()),
        original_grade: Set(original_grade),
        original_scheme: Set(original_scheme.map(|s| s.to_string())),
        is_final: Set(is_final),
        passed: Set(passed),
        attempt_number: Set(attempt_number),
        exam_date: Set(exam_date.clone()),
        recorded_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let grade_result = grade_model.insert(db).await?;
    let grade_id = grade_result.id;

    // Create component entries
    let mut component_infos = Vec::new();
    for comp in components {
        let component_model = grade_components::ActiveModel {
            course_id: Set(course_id),
            grade_id: Set(Some(grade_id)),
            component_name: Set(comp.name.clone()),
            weight: Set(comp.weight),
            points_earned: Set(comp.points_earned),
            points_total: Set(comp.points_total),
            grade: Set(comp.grade),
            is_bonus: Set(comp.is_bonus),
            bonus_points: Set(comp.bonus_points),
            is_completed: Set(true),
            due_date: Set(None),
            completed_at: Set(Some(Utc::now())),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let component_result = component_model.insert(db).await?;

        component_infos.push(ComponentInfo {
            id: component_result.id,
            component_name: comp.name,
            weight: comp.weight,
            points_earned: comp.points_earned,
            points_total: comp.points_total,
            grade: comp.grade,
            is_bonus: comp.is_bonus,
            bonus_points: comp.bonus_points,
            is_completed: true,
        });
    }

    Ok(GradeInfo {
        id: grade_id,
        course_id,
        grade,
        grading_scheme,
        original_grade,
        original_scheme,
        is_final,
        passed,
        attempt_number,
        exam_date,
        recorded_at: grade_result.recorded_at.to_rfc3339(),
        components: component_infos,
    })
}

/// Get a grade by ID with all its components
pub async fn get_grade_by_id(db: &DatabaseConnection, grade_id: i64) -> Result<GradeInfo> {
    // Get grade
    let grade = Grades::find_by_id(grade_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::NotFound(format!("Grade with ID {} not found", grade_id)))?;

    // Get components
    let components = GradeComponents::find()
        .filter(grade_components::Column::GradeId.eq(grade_id))
        .all(db)
        .await?;

    let component_infos: Vec<ComponentInfo> = components
        .into_iter()
        .map(|c| ComponentInfo {
            id: c.id,
            component_name: c.component_name,
            weight: c.weight,
            points_earned: c.points_earned,
            points_total: c.points_total,
            grade: c.grade,
            is_bonus: c.is_bonus,
            bonus_points: c.bonus_points,
            is_completed: c.is_completed,
        })
        .collect();

    Ok(GradeInfo {
        id: grade.id,
        course_id: grade.course_id,
        grade: grade.grade,
        grading_scheme: GradingScheme::from_str(&grade.grading_scheme)
            .unwrap_or(GradingScheme::German),
        original_grade: grade.original_grade,
        original_scheme: grade
            .original_scheme
            .and_then(|s| GradingScheme::from_str(&s)),
        is_final: grade.is_final,
        passed: grade.passed,
        attempt_number: grade.attempt_number,
        exam_date: grade.exam_date,
        recorded_at: grade.recorded_at.to_rfc3339(),
        components: component_infos,
    })
}

/// List all grades for a course
pub async fn list_grades_by_course(
    db: &DatabaseConnection,
    course_id: i64,
) -> Result<Vec<GradeInfo>> {
    let grades = Grades::find()
        .filter(grades::Column::CourseId.eq(course_id))
        .order_by_desc(grades::Column::RecordedAt)
        .all(db)
        .await?;

    let mut grade_infos = Vec::new();
    for grade in grades {
        let info = get_grade_by_id(db, grade.id).await?;
        grade_infos.push(info);
    }

    Ok(grade_infos)
}

/// Get the final grade for a course (most recent final grade)
pub async fn get_final_grade(db: &DatabaseConnection, course_id: i64) -> Result<Option<GradeInfo>> {
    let grade = Grades::find()
        .filter(grades::Column::CourseId.eq(course_id))
        .filter(grades::Column::IsFinal.eq(true))
        .order_by_desc(grades::Column::RecordedAt)
        .one(db)
        .await?;

    match grade {
        Some(g) => Ok(Some(get_grade_by_id(db, g.id).await?)),
        None => Ok(None),
    }
}

/// Update a grade
pub async fn update_grade(
    db: &DatabaseConnection,
    grade_id: i64,
    new_grade: Option<f64>,
    new_scheme: Option<GradingScheme>,
    is_final: Option<bool>,
) -> Result<GradeInfo> {
    let mut grade: grades::ActiveModel = Grades::find_by_id(grade_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::NotFound(format!("Grade with ID {} not found", grade_id)))?
        .into();

    if let Some(g) = new_grade {
        grade.grade = Set(g);
        // Recalculate passed status
        let scheme = if let Some(s) = new_scheme {
            s
        } else {
            GradingScheme::from_str(&grade.grading_scheme.clone().unwrap())
                .unwrap_or(GradingScheme::German)
        };
        grade.passed = Set(scheme.is_passing(g));
    }

    if let Some(s) = new_scheme {
        grade.grading_scheme = Set(s.to_string());
    }

    if let Some(f) = is_final {
        grade.is_final = Set(f);
    }

    grade.updated_at = Set(Utc::now());

    let updated = grade.update(db).await?;
    get_grade_by_id(db, updated.id).await
}

/// Delete a grade
pub async fn delete_grade(db: &DatabaseConnection, grade_id: i64) -> Result<()> {
    let res = Grades::delete_by_id(grade_id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(MmsError::NotFound(format!(
            "Grade with ID {} not found",
            grade_id
        )));
    }
    Ok(())
}

/// Add a component to an existing grade
pub async fn add_grade_component(
    db: &DatabaseConnection,
    grade_id: i64,
    component: ComponentDefinition,
) -> Result<ComponentInfo> {
    // Verify grade exists
    let grade = Grades::find_by_id(grade_id)
        .one(db)
        .await?
        .ok_or_else(|| MmsError::NotFound(format!("Grade with ID {} not found", grade_id)))?;

    let component_model = grade_components::ActiveModel {
        course_id: Set(grade.course_id),
        grade_id: Set(Some(grade_id)),
        component_name: Set(component.name.clone()),
        weight: Set(component.weight),
        points_earned: Set(component.points_earned),
        points_total: Set(component.points_total),
        grade: Set(component.grade),
        is_bonus: Set(component.is_bonus),
        bonus_points: Set(component.bonus_points),
        is_completed: Set(true),
        due_date: Set(None),
        completed_at: Set(Some(Utc::now())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let result = component_model.insert(db).await?;

    Ok(ComponentInfo {
        id: result.id,
        component_name: component.name,
        weight: component.weight,
        points_earned: component.points_earned,
        points_total: component.points_total,
        grade: component.grade,
        is_bonus: component.is_bonus,
        bonus_points: component.bonus_points,
        is_completed: true,
    })
}

/// Delete a grade component
pub async fn delete_grade_component(db: &DatabaseConnection, component_id: i64) -> Result<()> {
    let res = GradeComponents::delete_by_id(component_id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(MmsError::NotFound(format!(
            "Grade component with ID {} not found",
            component_id
        )));
    }
    Ok(())
}

/// Get all grades that are marked as final
pub async fn list_final_grades(db: &DatabaseConnection) -> Result<Vec<GradeInfo>> {
    let grades = Grades::find()
        .filter(grades::Column::IsFinal.eq(true))
        .order_by_desc(grades::Column::RecordedAt)
        .all(db)
        .await?;

    let mut grade_infos = Vec::new();
    for grade in grades {
        let info = get_grade_by_id(db, grade.id).await?;
        grade_infos.push(info);
    }

    Ok(grade_infos)
}

/// Get all passing grades
pub async fn list_passing_grades(db: &DatabaseConnection) -> Result<Vec<GradeInfo>> {
    let grades = Grades::find()
        .filter(grades::Column::Passed.eq(true))
        .order_by_desc(grades::Column::RecordedAt)
        .all(db)
        .await?;

    let mut grade_infos = Vec::new();
    for grade in grades {
        let info = get_grade_by_id(db, grade.id).await?;
        grade_infos.push(info);
    }

    Ok(grade_infos)
}
