pub mod builder;
pub mod operations;

pub use builder::CourseBuilder;
pub use operations::{
    create_course, update_course, delete_course,
    get_course_by_id, get_course_by_short_name, list_courses,
    CourseInfo,
};
