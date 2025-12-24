use crate::db::entities::{prelude::Todos, todos};
use crate::error::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn insert(db: &DatabaseConnection, todo: todos::ActiveModel) -> Result<i64> {
    let res = todo.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<todos::Model> {
    let todo = Todos::find_by_id(id).one(db).await?.ok_or_else(|| {
        crate::error::MmsError::NotFound(format!("Todo with ID {} not found", id))
    })?;
    Ok(todo)
}

pub async fn list_by_course(db: &DatabaseConnection, course_id: i64) -> Result<Vec<todos::Model>> {
    let todos = Todos::find()
        .filter(todos::Column::CourseId.eq(course_id))
        .all(db)
        .await?;
    Ok(todos)
}

pub async fn update(db: &DatabaseConnection, todo: todos::ActiveModel) -> Result<todos::Model> {
    let todo = todo.update(db).await?;
    Ok(todo)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = Todos::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!(
            "Todo with ID {} not found",
            id
        )));
    }
    Ok(())
}
