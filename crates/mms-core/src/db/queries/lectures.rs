use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use crate::error::Result;
use crate::db::entities::{lectures, prelude::Lectures};

pub async fn insert(db: &DatabaseConnection, lecture: lectures::ActiveModel) -> Result<i64> {
    let res = lecture.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<lectures::Model> {
    let lecture = Lectures::find_by_id(id).one(db).await?
        .ok_or_else(|| crate::error::MmsError::NotFound(format!("Lecture with ID {} not found", id)))?;
    Ok(lecture)
}

pub async fn list_by_course(db: &DatabaseConnection, course_id: i64) -> Result<Vec<lectures::Model>> {
    let lectures = Lectures::find()
        .filter(lectures::Column::CourseId.eq(course_id))
        .order_by_asc(lectures::Column::LectureNumber)
        .all(db).await?;
    Ok(lectures)
}

pub async fn update(db: &DatabaseConnection, lecture: lectures::ActiveModel) -> Result<lectures::Model> {
    let lecture = lecture.update(db).await?;
    Ok(lecture)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = Lectures::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!("Lecture with ID {} not found", id)));
    }
    Ok(())
}
