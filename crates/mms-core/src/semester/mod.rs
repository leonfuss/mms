pub mod builder;
pub mod operations;

pub use builder::SemesterBuilder;
pub use operations::{
    SemesterInfo, create_semester, delete_semester, get_semester_by_code, get_semester_by_id,
    list_semesters, update_semester,
};

/// Re-export SemesterType for convenience
pub use crate::toml::SemesterType;
