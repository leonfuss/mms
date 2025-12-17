use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use crate::error::Result;
use crate::db::entities::{course_possible_categories, course_degree_mappings, prelude::{CoursePossibleCategories, CourseDegreeMappings}};

// CoursePossibleCategory Queries

pub async fn insert_possible_category(db: &DatabaseConnection, category: course_possible_categories::ActiveModel) -> Result<i64> {
    let res = category.insert(db).await?;
    Ok(res.id)
}

pub async fn list_possible_categories_by_course(db: &DatabaseConnection, course_id: i64) -> Result<Vec<course_possible_categories::Model>> {
    let categories = CoursePossibleCategories::find()
        .filter(course_possible_categories::Column::CourseId.eq(course_id))
        .all(db).await?;
    Ok(categories)
}

// CourseDegreeMapping Queries

pub async fn insert_degree_mapping(db: &DatabaseConnection, mapping: course_degree_mappings::ActiveModel) -> Result<i64> {
    let res = mapping.insert(db).await?;
    Ok(res.id)
}

pub async fn get_degree_mapping_by_id(db: &DatabaseConnection, id: i64) -> Result<course_degree_mappings::Model> {
    let mapping = CourseDegreeMappings::find_by_id(id).one(db).await?
        .ok_or_else(|| crate::error::MmsError::NotFound(format!("CourseDegreeMapping with ID {} not found", id)))?;
    Ok(mapping)
}

pub async fn list_degree_mappings_by_course(db: &DatabaseConnection, course_id: i64) -> Result<Vec<course_degree_mappings::Model>> {
    let mappings = CourseDegreeMappings::find()
        .filter(course_degree_mappings::Column::CourseId.eq(course_id))
        .all(db).await?;
    Ok(mappings)
}

pub async fn delete_degree_mapping(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = CourseDegreeMappings::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!("CourseDegreeMapping with ID {} not found", id)));
    }
    Ok(())
}
