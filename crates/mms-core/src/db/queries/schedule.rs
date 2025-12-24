use crate::db::entities::{course_schedules, prelude::CourseSchedules};
use crate::error::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};

pub async fn insert(
    db: &DatabaseConnection,
    schedule: course_schedules::ActiveModel,
) -> Result<i64> {
    let res = schedule.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<course_schedules::Model> {
    let schedule = CourseSchedules::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| {
            crate::error::MmsError::NotFound(format!("CourseSchedule with ID {} not found", id))
        })?;
    Ok(schedule)
}

pub async fn list_by_course(
    db: &DatabaseConnection,
    course_id: i64,
) -> Result<Vec<course_schedules::Model>> {
    let schedules = CourseSchedules::find()
        .filter(course_schedules::Column::CourseId.eq(course_id))
        .order_by_asc(course_schedules::Column::DayOfWeek)
        .order_by_asc(course_schedules::Column::StartTime)
        .all(db)
        .await?;
    Ok(schedules)
}

pub async fn update(
    db: &DatabaseConnection,
    schedule: course_schedules::ActiveModel,
) -> Result<course_schedules::Model> {
    let schedule = schedule.update(db).await?;
    Ok(schedule)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = CourseSchedules::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!(
            "CourseSchedule with ID {} not found",
            id
        )));
    }
    Ok(())
}
