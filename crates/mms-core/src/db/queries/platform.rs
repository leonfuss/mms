use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use crate::error::Result;
use crate::db::entities::{platform_accounts, platform_course_links, prelude::{PlatformAccounts, PlatformCourseLinks}};

// PlatformAccount Queries

pub async fn insert_platform_account(db: &DatabaseConnection, account: platform_accounts::ActiveModel) -> Result<i64> {
    let res = account.insert(db).await?;
    Ok(res.id)
}

pub async fn get_platform_account_by_id(db: &DatabaseConnection, id: i64) -> Result<platform_accounts::Model> {
    let account = PlatformAccounts::find_by_id(id).one(db).await?
        .ok_or_else(|| crate::error::MmsError::NotFound(format!("PlatformAccount with ID {} not found", id)))?;
    Ok(account)
}

pub async fn list_platform_accounts(db: &DatabaseConnection) -> Result<Vec<platform_accounts::Model>> {
    let accounts = PlatformAccounts::find().all(db).await?;
    Ok(accounts)
}

pub async fn update_platform_account(db: &DatabaseConnection, account: platform_accounts::ActiveModel) -> Result<platform_accounts::Model> {
    let account = account.update(db).await?;
    Ok(account)
}

pub async fn delete_platform_account(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = PlatformAccounts::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!("PlatformAccount with ID {} not found", id)));
    }
    Ok(())
}

// PlatformCourseLink Queries

pub async fn insert_platform_course_link(db: &DatabaseConnection, link: platform_course_links::ActiveModel) -> Result<i64> {
    let res = link.insert(db).await?;
    Ok(res.id)
}

pub async fn get_platform_course_link_by_id(db: &DatabaseConnection, id: i64) -> Result<platform_course_links::Model> {
    let link = PlatformCourseLinks::find_by_id(id).one(db).await?
        .ok_or_else(|| crate::error::MmsError::NotFound(format!("PlatformCourseLink with ID {} not found", id)))?;
    Ok(link)
}

pub async fn list_platform_course_links_by_course(db: &DatabaseConnection, course_id: i64) -> Result<Vec<platform_course_links::Model>> {
    let links = PlatformCourseLinks::find()
        .filter(platform_course_links::Column::CourseId.eq(course_id))
        .all(db).await?;
    Ok(links)
}

pub async fn update_platform_course_link(db: &DatabaseConnection, link: platform_course_links::ActiveModel) -> Result<platform_course_links::Model> {
    let link = link.update(db).await?;
    Ok(link)
}

pub async fn delete_platform_course_link(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = PlatformCourseLinks::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!("PlatformCourseLink with ID {} not found", id)));
    }
    Ok(())
}
