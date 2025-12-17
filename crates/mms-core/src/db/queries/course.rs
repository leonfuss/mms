use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use crate::error::Result;
use crate::db::entities::{courses, prelude::Courses};

pub async fn insert(db: &DatabaseConnection, course: courses::ActiveModel) -> Result<i64> {
    let res = course.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<courses::Model> {
    let course = Courses::find_by_id(id).one(db).await?
        .ok_or_else(|| crate::error::MmsError::NotFound(format!("Course with ID {} not found", id)))?;
    Ok(course)
}

pub async fn list(db: &DatabaseConnection) -> Result<Vec<courses::Model>> {
    let courses = Courses::find()
        .order_by_desc(courses::Column::SemesterId)
        .order_by_asc(courses::Column::ShortName)
        .all(db).await?;
    Ok(courses)
}

pub async fn list_by_semester(db: &DatabaseConnection, semester_id: i64) -> Result<Vec<courses::Model>> {
    let courses = Courses::find()
        .filter(courses::Column::SemesterId.eq(semester_id))
        .order_by_asc(courses::Column::ShortName)
        .all(db).await?;
    Ok(courses)
}

pub async fn update(db: &DatabaseConnection, course: courses::ActiveModel) -> Result<courses::Model> {
    let course = course.update(db).await?;
    Ok(course)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = Courses::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!("Course with ID {} not found", id)));
    }
    Ok(())
}
