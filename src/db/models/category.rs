use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRequirement {
    pub id: Option<i64>,
    pub category_name: String,
    pub required_ects: i32,
    pub counts_towards_average: bool,
}

impl CategoryRequirement {
    pub fn new(category_name: String, required_ects: i32, counts_towards_average: bool) -> Self {
        Self {
            id: None,
            category_name,
            required_ects,
            counts_towards_average,
        }
    }
}
