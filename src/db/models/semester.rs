use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semester {
    pub id: Option<i64>,
    pub name: String,
    pub type_: SemesterType,
    pub number: i32,
    pub is_current: bool,
    pub default_location: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemesterType {
    Bachelor,
    Master,
}

impl SemesterType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bachelor" | "b" => Some(SemesterType::Bachelor),
            "master" | "m" => Some(SemesterType::Master),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            SemesterType::Bachelor => "bachelor",
            SemesterType::Master => "master",
        }
    }

    pub fn initial(&self) -> char {
        match self {
            SemesterType::Bachelor => 'b',
            SemesterType::Master => 'm',
        }
    }
}

impl Semester {
    pub fn new(name: String, type_: SemesterType, number: i32, default_location: Option<String>) -> Self {
        Self {
            id: None,
            name,
            type_,
            number,
            is_current: false,
            default_location: default_location.unwrap_or_else(|| "Uni Tübingen".to_string()),
            created_at: None,
        }
    }
}
