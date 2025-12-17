use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Active {
    pub id: i64, // Always 1, enforced by CHECK constraint
    pub semester_id: Option<i64>,
    pub course_id: Option<i64>,
    pub lecture_id: Option<i64>,
    pub activated_at: Option<NaiveDateTime>,
}

impl Active {
    pub fn new() -> Self {
        Self {
            id: 1,
            semester_id: None,
            course_id: None,
            lecture_id: None,
            activated_at: None,
        }
    }
}
