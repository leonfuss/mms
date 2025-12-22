pub mod builder;
pub mod operations;

pub use builder::SemesterBuilder;
pub use operations::{
    create_semester, update_semester, delete_semester,
    get_semester_by_id, get_semester_by_code, list_semesters,
    SemesterInfo,
};

/// Re-export SemesterType for convenience
pub use crate::toml::SemesterType;
