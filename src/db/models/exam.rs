use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exam {
    pub id: Option<i64>,
    pub course_id: i64,
    pub exam_type: ExamType,
    pub date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub room: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExamType {
    Written,
    Oral,
    Project,
}

impl ExamType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "written" => Some(ExamType::Written),
            "oral" => Some(ExamType::Oral),
            "project" => Some(ExamType::Project),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            ExamType::Written => "written",
            ExamType::Oral => "oral",
            ExamType::Project => "project",
        }
    }
}

impl Exam {
    pub fn new(course_id: i64, exam_type: ExamType, date: NaiveDate) -> Self {
        Self {
            id: None,
            course_id,
            exam_type,
            date,
            start_time: None,
            end_time: None,
            room: None,
            location: None,
            notes: None,
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

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }
}
