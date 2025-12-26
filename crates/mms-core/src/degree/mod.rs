pub mod builder;
pub mod operations;
pub mod types;

pub use builder::DegreeBuilder;
pub use operations::{
    DegreeAreaInfo, DegreeInfo, DegreeProgressInfo, add_degree_area, create_degree, delete_degree,
    delete_degree_area, get_degree_by_id, get_degree_progress, get_unmapped_courses, list_degrees,
    map_course_to_area, unmap_course_from_area, update_degree, update_degree_area,
};
pub use types::{AreaEcts, DegreeEcts, DegreeType};
