use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holiday {
    pub id: Option<i64>,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub applies_to_semester_id: Option<i64>,
    pub created_at: Option<NaiveDateTime>,
}

impl Holiday {
    pub fn new(name: String, start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            id: None,
            name,
            start_date,
            end_date,
            applies_to_semester_id: None,
            created_at: None,
        }
    }

    pub fn for_semester(mut self, semester_id: i64) -> Self {
        self.applies_to_semester_id = Some(semester_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayException {
    pub id: Option<i64>,
    pub holiday_id: i64,
    pub course_schedule_id: i64,
    pub exception_date: NaiveDate,
}

impl HolidayException {
    pub fn new(holiday_id: i64, course_schedule_id: i64, exception_date: NaiveDate) -> Self {
        Self {
            id: None,
            holiday_id,
            course_schedule_id,
            exception_date,
        }
    }
}
