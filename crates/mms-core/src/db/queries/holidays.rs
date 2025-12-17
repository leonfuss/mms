use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use crate::error::Result;
use crate::db::entities::{holidays, holiday_exceptions, prelude::{Holidays, HolidayExceptions}};

// Holiday Queries

pub async fn insert_holiday(db: &DatabaseConnection, holiday: holidays::ActiveModel) -> Result<i64> {
    let res = holiday.insert(db).await?;
    Ok(res.id)
}

pub async fn get_holiday_by_id(db: &DatabaseConnection, id: i64) -> Result<holidays::Model> {
    let holiday = Holidays::find_by_id(id).one(db).await?
        .ok_or_else(|| crate::error::MmsError::NotFound(format!("Holiday with ID {} not found", id)))?;
    Ok(holiday)
}

pub async fn list_holidays(db: &DatabaseConnection) -> Result<Vec<holidays::Model>> {
    let holidays = Holidays::find()
        .order_by_asc(holidays::Column::StartDate)
        .all(db).await?;
    Ok(holidays)
}

pub async fn update_holiday(db: &DatabaseConnection, holiday: holidays::ActiveModel) -> Result<holidays::Model> {
    let holiday = holiday.update(db).await?;
    Ok(holiday)
}

pub async fn delete_holiday(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = Holidays::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!("Holiday with ID {} not found", id)));
    }
    Ok(())
}

// HolidayException Queries

pub async fn insert_holiday_exception(db: &DatabaseConnection, exception: holiday_exceptions::ActiveModel) -> Result<i64> {
    let res = exception.insert(db).await?;
    Ok(res.id)
}

pub async fn list_holiday_exceptions_by_holiday(db: &DatabaseConnection, holiday_id: i64) -> Result<Vec<holiday_exceptions::Model>> {
    let exceptions = HolidayExceptions::find()
        .filter(holiday_exceptions::Column::HolidayId.eq(holiday_id))
        .all(db).await?;
    Ok(exceptions)
}

pub async fn delete_holiday_exception(db: &DatabaseConnection, id: i64) -> Result<()> {
    let res = HolidayExceptions::delete_by_id(id).exec(db).await?;
    if res.rows_affected == 0 {
        return Err(crate::error::MmsError::NotFound(format!("HolidayException with ID {} not found", id)));
    }
    Ok(())
}
