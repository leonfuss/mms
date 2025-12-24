pub mod active;
pub mod course;
pub mod course_categories;
pub mod degree_areas;
pub mod degrees;
pub mod event;
pub mod exam_attempts; // Renamed from exam
pub mod exercises;
pub mod grades;
pub mod holidays;
pub mod lectures;
pub mod platform;
pub mod schedule;
pub mod semester;
pub mod slides;
pub mod todos;

// Re-export common types
pub use sea_orm::prelude::*; // This pulls in all traits like EntityTrait, ActiveModelTrait etc.
