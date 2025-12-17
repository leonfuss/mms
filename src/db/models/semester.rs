use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semester {
    pub id: Option<i64>,
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
            SemesterType::Bachelor => "Bachelor",
            SemesterType::Master => "Master",
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
    pub fn new(type_: SemesterType, number: i32, default_location: Option<String>) -> Self {
        Self {
            id: None,
            type_,
            number,
            is_current: false,
            default_location: default_location.unwrap_or_else(|| "Uni Tübingen".to_string()),
            created_at: None,
        }
    }

    /// Get the display name (e.g., "WS2425", "SS25")
    pub fn display_name(&self) -> String {
        format!("{} Semester {}", self.type_.to_str(), self.number)
    }

    /// Get the folder name (e.g., "m01", "b03")
    pub fn folder_name(&self) -> String {
        format!("{}{:02}", self.type_.initial(), self.number)
    }
}

impl fmt::Display for Semester {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
