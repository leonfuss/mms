use crate::db::entities::{exercises, prelude::Exercises};
use crate::error::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};

pub async fn insert(db: &DatabaseConnection, exercise: exercises::ActiveModel) -> Result<i64> {
    let res = exercise.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<exercises::Model> {
    let exercise = Exercises::find_by_id(id).one(db).await?.ok_or_else(|| {
        crate::error::MmsError::NotFound(format!("Exercise with ID {} not found", id))
    })?;
    Ok(exercise)
}

pub async fn list_by_course(
    db: &DatabaseConnection,
    course_id: i64,
) -> Result<Vec<exercises::Model>> {
    let exercises = Exercises::find()
        .filter(exercises::Column::CourseId.eq(course_id))
        .order_by_asc(exercises::Column::ExerciseNumber)
        .all(db)
        .await?;
    Ok(exercises)
}

pub async fn update(
    db: &DatabaseConnection,
    exercise: exercises::ActiveModel,
) -> Result<exercises::Model> {
    let exercise = exercise.update(db).await?;
    Ok(exercise)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = Exercises::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!(
            "Exercise with ID {} not found",
            id
        )));
    }
    Ok(())
}
