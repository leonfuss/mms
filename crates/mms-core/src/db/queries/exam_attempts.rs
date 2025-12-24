use crate::db::entities::{exam_attempts, prelude::ExamAttempts};
use crate::error::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};

pub async fn insert(
    db: &DatabaseConnection,
    exam_attempt: exam_attempts::ActiveModel,
) -> Result<i64> {
    let res = exam_attempt.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<exam_attempts::Model> {
    let exam_attempt = ExamAttempts::find_by_id(id).one(db).await?.ok_or_else(|| {
        crate::error::MmsError::NotFound(format!("ExamAttempt with ID {} not found", id))
    })?;
    Ok(exam_attempt)
}

pub async fn list_by_course(
    db: &DatabaseConnection,
    course_id: i64,
) -> Result<Vec<exam_attempts::Model>> {
    let exam_attempts = ExamAttempts::find()
        .filter(exam_attempts::Column::CourseId.eq(course_id))
        .order_by_asc(exam_attempts::Column::AttemptNumber)
        .all(db)
        .await?;
    Ok(exam_attempts)
}

pub async fn update(
    db: &DatabaseConnection,
    exam_attempt: exam_attempts::ActiveModel,
) -> Result<exam_attempts::Model> {
    let exam_attempt = exam_attempt.update(db).await?;
    Ok(exam_attempt)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = ExamAttempts::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!(
            "ExamAttempt with ID {} not found",
            id
        )));
    }
    Ok(())
}
