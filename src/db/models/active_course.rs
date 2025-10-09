use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCourse {
    pub id: i64, // Always 1, enforced by CHECK constraint
    pub course_id: Option<i64>,
    pub updated_at: NaiveDateTime,
}

impl ActiveCourse {
    pub fn new(course_id: Option<i64>) -> Self {
        Self {
            id: 1,
            course_id,
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}
