use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: Option<i64>,
    pub semester_id: i64,
    pub name: String,
    pub short_name: String,
    pub category: String,
    pub ects: i32,
    pub lecturer: Option<String>,
    pub learning_platform_url: Option<String>,
    pub location: Option<String>,
    pub grade: Option<f32>,
    pub counts_towards_average: bool,
    pub created_at: Option<NaiveDateTime>,
}

impl Course {
    pub fn new(
        semester_id: i64,
        name: String,
        short_name: String,
        category: String,
        ects: i32,
    ) -> Self {
        Self {
            id: None,
            semester_id,
            name,
            short_name,
            category,
            ects,
            lecturer: None,
            learning_platform_url: None,
            location: None,
            grade: None,
            counts_towards_average: true,
            created_at: None,
        }
    }

    pub fn with_lecturer(mut self, lecturer: String) -> Self {
        self.lecturer = Some(lecturer);
        self
    }

    pub fn with_learning_platform(mut self, url: String) -> Self {
        self.learning_platform_url = Some(url);
        self
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_grade(mut self, grade: f32) -> Self {
        self.grade = Some(grade);
        self
    }

    pub fn set_counts_towards_average(mut self, counts: bool) -> Self {
        self.counts_towards_average = counts;
        self
    }

    /// Get the folder name for this course (uses short_name)
    pub fn folder_name(&self) -> &str {
        &self.short_name
    }
}
