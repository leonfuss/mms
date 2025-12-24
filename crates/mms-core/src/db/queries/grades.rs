use crate::db::entities::{
    grade_components, grades,
    prelude::{GradeComponents, Grades},
};
use crate::error::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

// Grade Queries

pub async fn insert_grade(db: &DatabaseConnection, grade: grades::ActiveModel) -> Result<i64> {
    let res = grade.insert(db).await?;
    Ok(res.id)
}

pub async fn get_grade_by_id(db: &DatabaseConnection, id: i64) -> Result<grades::Model> {
    let grade = Grades::find_by_id(id).one(db).await?.ok_or_else(|| {
        crate::error::MmsError::NotFound(format!("Grade with ID {} not found", id))
    })?;
    Ok(grade)
}

pub async fn list_grades_by_course(
    db: &DatabaseConnection,
    course_id: i64,
) -> Result<Vec<grades::Model>> {
    let grades = Grades::find()
        .filter(grades::Column::CourseId.eq(course_id))
        .all(db)
        .await?;
    Ok(grades)
}

pub async fn update_grade(
    db: &DatabaseConnection,
    grade: grades::ActiveModel,
) -> Result<grades::Model> {
    let grade = grade.update(db).await?;
    Ok(grade)
}

pub async fn delete_grade(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = Grades::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!(
            "Grade with ID {} not found",
            id
        )));
    }
    Ok(())
}

// GradeComponent Queries

pub async fn insert_grade_component(
    db: &DatabaseConnection,
    component: grade_components::ActiveModel,
) -> Result<i64> {
    let res = component.insert(db).await?;
    Ok(res.id)
}

pub async fn list_grade_components_by_grade(
    db: &DatabaseConnection,
    grade_id: i64,
) -> Result<Vec<grade_components::Model>> {
    let components = GradeComponents::find()
        .filter(grade_components::Column::GradeId.eq(grade_id))
        .all(db)
        .await?;
    Ok(components)
}

pub async fn update_grade_component(
    db: &DatabaseConnection,
    component: grade_components::ActiveModel,
) -> Result<grade_components::Model> {
    let component = component.update(db).await?;
    Ok(component)
}

pub async fn delete_grade_component(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = GradeComponents::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!(
            "GradeComponent with ID {} not found",
            id
        )));
    }
    Ok(())
}
