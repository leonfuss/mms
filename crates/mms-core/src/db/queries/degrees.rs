use crate::db::entities::{degrees, prelude::Degrees};
use crate::error::Result;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryOrder};

pub async fn insert(db: &DatabaseConnection, degree: degrees::ActiveModel) -> Result<i64> {
    let res = degree.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<degrees::Model> {
    let degree = Degrees::find_by_id(id).one(db).await?.ok_or_else(|| {
        crate::error::MmsError::NotFound(format!("Degree with ID {} not found", id))
    })?;
    Ok(degree)
}

pub async fn list(db: &DatabaseConnection) -> Result<Vec<degrees::Model>> {
    let degrees = Degrees::find()
        .order_by_asc(degrees::Column::Type)
        .order_by_asc(degrees::Column::Name)
        .all(db)
        .await?;
    Ok(degrees)
}

pub async fn update(
    db: &DatabaseConnection,
    degree: degrees::ActiveModel,
) -> Result<degrees::Model> {
    let degree = degree.update(db).await?;
    Ok(degree)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = Degrees::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!(
            "Degree with ID {} not found",
            id
        )));
    }
    Ok(())
}
