use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lecture {
    pub id: Option<i64>,
    pub course_id: i64,
    pub lecture_number: i32,
    pub date: NaiveDate,
    pub git_commit_hash: Option<String>,
}

impl Lecture {
    pub fn new(course_id: i64, lecture_number: i32, date: NaiveDate) -> Self {
        Self {
            id: None,
            course_id,
            lecture_number,
            date,
            git_commit_hash: None,
        }
    }

    pub fn with_commit_hash(mut self, hash: String) -> Self {
        self.git_commit_hash = Some(hash);
        self
    }
}
