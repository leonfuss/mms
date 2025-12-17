use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use crate::error::Result;
use crate::db::entities::{semesters, prelude::Semesters};
use chrono::Utc;
use sea_orm::sea_query::Expr;

pub async fn insert(db: &DatabaseConnection, semester: semesters::ActiveModel) -> Result<i64> {
    let res = semester.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<semesters::Model> {
    let semester = Semesters::find_by_id(id).one(db).await?
        .ok_or_else(|| crate::error::MmsError::NotFound(format!("Semester with ID {} not found", id)))?;
    Ok(semester)
}

pub async fn list(db: &DatabaseConnection) -> Result<Vec<semesters::Model>> {
    let semesters = Semesters::find()
        .order_by_desc(semesters::Column::Type)
        .order_by_desc(semesters::Column::Number)
        .all(db).await?;
    Ok(semesters)
}

pub async fn get_current(db: &DatabaseConnection) -> Result<Option<semesters::Model>> {
    let semester = Semesters::find()
        .filter(semesters::Column::IsCurrent.eq(true))
        .one(db).await?;
    Ok(semester)
}

pub async fn set_current(db: &DatabaseConnection, id: i64) -> Result<()> {
    // Unset all semesters
    semesters::Entity::update_many()
        .col_expr(semesters::Column::IsCurrent, Expr::value(false))
        .exec(db).await?;

    // Set the specified semester as current
    let mut semester: semesters::ActiveModel = Semesters::find_by_id(id).one(db).await?
        .ok_or_else(|| crate::error::MmsError::NotFound(format!("Semester with ID {} not found", id)))?
        .into();
    
    semester.is_current = ActiveValue::Set(true);
    semester.updated_at = ActiveValue::Set(Utc::now()); // Changed to Utc::now()

    semester.update(db).await?;

    Ok(())
}

pub async fn update(db: &DatabaseConnection, semester: semesters::ActiveModel) -> Result<semesters::Model> {
    let semester = semester.update(db).await?;
    Ok(semester)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = Semesters::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!("Semester with ID {} not found", id)));
    }
    Ok(())
}