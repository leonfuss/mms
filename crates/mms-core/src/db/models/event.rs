use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

use super::schedule::ScheduleType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseEvent {
    pub id: Option<i64>,
    pub course_id: i64,
    pub course_schedule_id: Option<i64>,
    pub schedule_type: ScheduleType,
    pub event_type: EventType,
    pub date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub room: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    OneTime,
    Makeup,
    Special,
    Override,
    Cancelled,
}

impl EventType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "one-time" | "onetime" | "OneTime" => Some(EventType::OneTime),
            "makeup" | "Makeup" => Some(EventType::Makeup),
            "special" | "Special" => Some(EventType::Special),
            "override" | "Override" => Some(EventType::Override),
            "cancelled" | "Cancelled" => Some(EventType::Cancelled),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            EventType::OneTime => "OneTime",
            EventType::Makeup => "Makeup",
            EventType::Special => "Special",
            EventType::Override => "Override",
            EventType::Cancelled => "Cancelled",
        }
    }
}

impl CourseEvent {
    pub fn new_one_time(
        course_id: i64,
        schedule_type: ScheduleType,
        date: NaiveDate,
        start_time: NaiveTime,
        end_time: NaiveTime,
    ) -> Self {
        Self {
            id: None,
            course_id,
            course_schedule_id: None,
            schedule_type,
            event_type: EventType::OneTime,
            date,
            start_time: Some(start_time),
            end_time: Some(end_time),
            room: None,
            location: None,
            description: None,
        }
    }

    pub fn new_cancelled(course_schedule_id: i64, course_id: i64, date: NaiveDate) -> Self {
        Self {
            id: None,
            course_id,
            course_schedule_id: Some(course_schedule_id),
            schedule_type: ScheduleType::Lecture, // Default, will be overridden
            event_type: EventType::Cancelled,
            date,
            start_time: None,
            end_time: None,
            room: None,
            location: None,
            description: None,
        }
    }

    pub fn new_override(
        course_schedule_id: i64,
        course_id: i64,
        schedule_type: ScheduleType,
        date: NaiveDate,
    ) -> Self {
        Self {
            id: None,
            course_id,
            course_schedule_id: Some(course_schedule_id),
            schedule_type,
            event_type: EventType::Override,
            date,
            start_time: None,
            end_time: None,
            room: None,
            location: None,
            description: None,
        }
    }

    pub fn with_time(mut self, start_time: NaiveTime, end_time: NaiveTime) -> Self {
        self.start_time = Some(start_time);
        self.end_time = Some(end_time);
        self
    }

    pub fn with_room(mut self, room: String) -> Self {
        self.room = Some(room);
        self
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}
