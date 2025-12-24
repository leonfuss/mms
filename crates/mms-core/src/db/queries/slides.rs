use crate::db::entities::{prelude::Slides, slides};
use crate::error::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};

pub async fn insert(db: &DatabaseConnection, slide: slides::ActiveModel) -> Result<i64> {
    let res = slide.insert(db).await?;
    Ok(res.id)
}

pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> Result<slides::Model> {
    let slide = Slides::find_by_id(id).one(db).await?.ok_or_else(|| {
        crate::error::MmsError::NotFound(format!("Slide with ID {} not found", id))
    })?;
    Ok(slide)
}

pub async fn list_by_course(db: &DatabaseConnection, course_id: i64) -> Result<Vec<slides::Model>> {
    let slides = Slides::find()
        .filter(slides::Column::CourseId.eq(course_id))
        .order_by_asc(slides::Column::FileName)
        .all(db)
        .await?;
    Ok(slides)
}

pub async fn update(db: &DatabaseConnection, slide: slides::ActiveModel) -> Result<slides::Model> {
    let slide = slide.update(db).await?;
    Ok(slide)
}

pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = Slides::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!(
            "Slide with ID {} not found",
            id
        )));
    }
    Ok(())
}
