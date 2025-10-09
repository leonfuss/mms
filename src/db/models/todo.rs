use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Option<i64>,
    pub course_id: i64,
    pub lecture_number: Option<i32>,
    pub description: String,
    pub completed: bool,
    pub auto_clear: bool,
    pub created_at: Option<NaiveDateTime>,
    pub cleared_at: Option<NaiveDateTime>,
}

impl Todo {
    pub fn new(course_id: i64, description: String) -> Self {
        Self {
            id: None,
            course_id,
            lecture_number: None,
            description,
            completed: false,
            auto_clear: true,
            created_at: None,
            cleared_at: None,
        }
    }

    pub fn for_lecture(mut self, lecture_number: i32) -> Self {
        self.lecture_number = Some(lecture_number);
        self
    }

    pub fn no_auto_clear(mut self) -> Self {
        self.auto_clear = false;
        self
    }

    pub fn mark_completed(&mut self) {
        self.completed = true;
        self.cleared_at = Some(chrono::Utc::now().naive_utc());
    }
}
