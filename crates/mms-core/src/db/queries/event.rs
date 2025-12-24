use crate::db::entities::{course_events, prelude::CourseEvents};
use crate::error::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};

pub async fn insert(db: &DatabaseConnection, event: course_events::ActiveModel) -> Result<i64> {
    let res = event.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<course_events::Model> {
    let event = CourseEvents::find_by_id(id).one(db).await?.ok_or_else(|| {
        crate::error::MmsError::NotFound(format!("CourseEvent with ID {} not found", id))
    })?;
    Ok(event)
}

pub async fn list_by_course(
    db: &DatabaseConnection,
    course_id: i64,
) -> Result<Vec<course_events::Model>> {
    let events = CourseEvents::find()
        .filter(course_events::Column::CourseId.eq(course_id))
        .order_by_asc(course_events::Column::Date)
        .order_by_asc(course_events::Column::StartTime)
        .all(db)
        .await?;
    Ok(events)
}

pub async fn get_by_course_and_date(
    db: &DatabaseConnection,
    course_id: i64,
    date: String,
) -> Result<Vec<course_events::Model>> {
    let events = CourseEvents::find()
        .filter(course_events::Column::CourseId.eq(course_id))
        .filter(course_events::Column::Date.eq(date))
        .all(db)
        .await?;
    Ok(events)
}

pub async fn update(
    db: &DatabaseConnection,
    event: course_events::ActiveModel,
) -> Result<course_events::Model> {
    let event = event.update(db).await?;
    Ok(event)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = CourseEvents::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!(
            "CourseEvent with ID {} not found",
            id
        )));
    }
    Ok(())
}
