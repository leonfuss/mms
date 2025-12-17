pub mod semester;
pub mod course;
pub mod active;
pub mod schedule;
pub mod event;
pub mod degrees;
pub mod degree_areas;
pub mod course_categories;
pub mod grades;
pub mod exam_attempts; // Renamed from exam
pub mod holidays;
pub mod lectures;
pub mod slides;
pub mod exercises;
pub mod todos;
pub mod platform;

// Re-export common types
pub use sea_orm::prelude::*; // This pulls in all traits like EntityTrait, ActiveModelTrait etc.