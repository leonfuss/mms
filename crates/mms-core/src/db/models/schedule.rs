use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSchedule {
    pub id: Option<i64>,
    pub course_id: i64,
    pub schedule_type: ScheduleType,
    pub day_of_week: u32, // 0=Monday, 6=Sunday
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub room: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleType {
    Lecture,
    Tutorium,
    Exercise,
}

impl ScheduleType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "lecture" | "Lecture" => Some(ScheduleType::Lecture),
            "tutorium" | "Tutorium" => Some(ScheduleType::Tutorium),
            "exercise" | "Exercise" => Some(ScheduleType::Exercise),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            ScheduleType::Lecture => "Lecture",
            ScheduleType::Tutorium => "Tutorium",
            ScheduleType::Exercise => "Exercise",
        }
    }
}

impl CourseSchedule {
    pub fn new(
        course_id: i64,
        schedule_type: ScheduleType,
        day_of_week: u32,
        start_time: NaiveTime,
        end_time: NaiveTime,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Self {
        Self {
            id: None,
            course_id,
            schedule_type,
            day_of_week,
            start_time,
            end_time,
            start_date,
            end_date,
            room: None,
            location: None,
        }
    }

    pub fn with_room(mut self, room: String) -> Self {
        self.room = Some(room);
        self
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }
}
