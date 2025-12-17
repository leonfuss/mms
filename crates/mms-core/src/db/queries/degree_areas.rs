use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use crate::error::Result;
use crate::db::entities::{degree_areas, prelude::DegreeAreas};

pub async fn insert(db: &DatabaseConnection, degree_area: degree_areas::ActiveModel) -> Result<i64> {
    let res = degree_area.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<degree_areas::Model> {
    let degree_area = DegreeAreas::find_by_id(id).one(db).await?
        .ok_or_else(|| crate::error::MmsError::NotFound(format!("DegreeArea with ID {} not found", id)))?;
    Ok(degree_area)
}

pub async fn list_by_degree(db: &DatabaseConnection, degree_id: i64) -> Result<Vec<degree_areas::Model>> {
    let areas = DegreeAreas::find()
        .filter(degree_areas::Column::DegreeId.eq(degree_id))
        .order_by_asc(degree_areas::Column::DisplayOrder)
        .all(db).await?;
    Ok(areas)
}

pub async fn update(db: &DatabaseConnection, degree_area: degree_areas::ActiveModel) -> Result<degree_areas::Model> {
    let degree_area = degree_area.update(db).await?;
    Ok(degree_area)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = DegreeAreas::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!("DegreeArea with ID {} not found", id)));
    }
    Ok(())
}
