use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait,
    Set,
};
use crate::error::Result;
use crate::db::entities::{active_course, prelude::ActiveCourse as ActiveCourseEntity};
use chrono::Utc;
use sea_orm::IntoActiveModel;

pub async fn get(db: &DatabaseConnection) -> Result<active_course::Model> {
    let active = ActiveCourseEntity::find_by_id(1).one(db).await?
        .ok_or_else(|| crate::error::MmsError::NotFound("ActiveCourse singleton not found.".to_string()))?;
    Ok(active)
}

pub async fn set_active_semester(db: &DatabaseConnection, semester_id: i64) -> Result<()> {
    let now = Utc::now(); // Changed to Utc::now()

    let mut active: active_course::ActiveModel = match ActiveCourseEntity::find_by_id(1).one(db).await? {
        Some(model) => model.into_active_model(),
        None => active_course::ActiveModel { id: ActiveValue::Set(1), ..Default::default() },
    };

    active.semester_id = Set(Some(semester_id));
    active.activated_at = Set(Some(now)); // Clone not needed for Copy/Utc
    active.updated_at = Set(now);
    
    // Save (insert or update)
    if active.id.is_set() && active.id.as_ref() == &1 && ActiveCourseEntity::find_by_id(1).one(db).await?.is_some() {
         active.update(db).await?;
    } else {
         active.insert(db).await?;
    }

    Ok(())
}

pub async fn set_active_course(db: &DatabaseConnection, course_id: i64, semester_id: i64) -> Result<()> {
    let now = Utc::now();

    let mut active: active_course::ActiveModel = match ActiveCourseEntity::find_by_id(1).one(db).await? {
        Some(model) => model.into_active_model(),
        None => active_course::ActiveModel { id: ActiveValue::Set(1), ..Default::default() },
    };

    active.semester_id = Set(Some(semester_id));
    active.course_id = Set(Some(course_id));
    active.lecture_id = Set(None);
    active.activated_at = Set(Some(now));
    active.updated_at = Set(now);

    let exists = ActiveCourseEntity::find_by_id(1).one(db).await?.is_some();
    if exists {
        active.update(db).await?;
    } else {
        active.insert(db).await?;
    }

    Ok(())
}

pub async fn clear_active_course(db: &DatabaseConnection) -> Result<()> {
    let now = Utc::now();

    let mut active: active_course::ActiveModel = match ActiveCourseEntity::find_by_id(1).one(db).await? {
        Some(model) => model.into_active_model(),
        None => active_course::ActiveModel { id: ActiveValue::Set(1), ..Default::default() },
    };

    active.course_id = Set(None);
    active.lecture_id = Set(None);
    active.updated_at = Set(now);

    let exists = ActiveCourseEntity::find_by_id(1).one(db).await?.is_some();
    if exists {
        active.update(db).await?;
    } else {
        active.insert(db).await?;
    }

    Ok(())
}

pub async fn clear_all(db: &DatabaseConnection) -> Result<()> {
    let now = Utc::now();

    let mut active: active_course::ActiveModel = match ActiveCourseEntity::find_by_id(1).one(db).await? {
        Some(model) => model.into_active_model(),
        None => active_course::ActiveModel { id: ActiveValue::Set(1), ..Default::default() },
    };

    active.semester_id = Set(None);
    active.course_id = Set(None);
    active.lecture_id = Set(None);
    active.activated_at = Set(None);
    active.updated_at = Set(now);

    let exists = ActiveCourseEntity::find_by_id(1).one(db).await?.is_some();
    if exists {
        active.update(db).await?;
    } else {
        active.insert(db).await?;
    }
    
    Ok(())
}

pub async fn set_active_lecture(db: &DatabaseConnection, lecture_id: i64) -> Result<()> {
    let now = Utc::now();

    let mut active: active_course::ActiveModel = match ActiveCourseEntity::find_by_id(1).one(db).await? {
        Some(model) => model.into_active_model(),
        None => active_course::ActiveModel { id: ActiveValue::Set(1), ..Default::default() },
    };

    active.lecture_id = Set(Some(lecture_id));
    active.updated_at = Set(now);

    let exists = ActiveCourseEntity::find_by_id(1).one(db).await?.is_some();
    if exists {
        active.update(db).await?;
    } else {
        active.insert(db).await?;
    }
    
    Ok(())
}