pub mod builder;
pub mod operations;
pub mod types;

pub use builder::CourseBuilder;
pub use operations::{
    CourseInfo, create_course, delete_course, get_course_by_id, get_course_by_short_name,
    list_courses, update_course,
};
pub use types::{CourseCode, Ects};
