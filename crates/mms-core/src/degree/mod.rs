pub mod builder;
pub mod operations;
pub mod types;

pub use builder::DegreeBuilder;
pub use operations::{
    create_degree, update_degree, delete_degree,
    get_degree_by_id, list_degrees,
    add_degree_area, update_degree_area, delete_degree_area,
    map_course_to_area, unmap_course_from_area,
    get_degree_progress, get_unmapped_courses,
    DegreeInfo, DegreeAreaInfo, DegreeProgressInfo,
};
pub use types::DegreeType;
