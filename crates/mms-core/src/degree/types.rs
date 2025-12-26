use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Degree type (Bachelor, Master, or PhD)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DegreeType {
    Bachelor,
    Master,
    #[serde(rename = "phd")]
    PhD,
}

impl FromStr for DegreeType {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bachelor" | "b" | "ba" | "bsc" => Ok(DegreeType::Bachelor),
            "master" | "m" | "ma" | "msc" => Ok(DegreeType::Master),
            "phd" | "doctorate" | "dr" => Ok(DegreeType::PhD),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for DegreeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DegreeType::Bachelor => write!(f, "bachelor"),
            DegreeType::Master => write!(f, "master"),
            DegreeType::PhD => write!(f, "phd"),
        }
    }
}

use crate::error::{MmsError, Result};
use std::fmt;
use std::ops::Deref;

/// Type-safe degree ECTS value
///
/// Validates ECTS based on degree type:
/// - PhD: 0 ECTS (no requirement)
/// - Bachelor: 90-240 ECTS
/// - Master: 60-120 ECTS
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DegreeEcts(i32);

impl DegreeEcts {
    /// Create a new DegreeEcts value with validation
    ///
    /// # Errors
    /// Returns `InvalidDegreeEcts` if value is not valid for the degree type
    pub fn new(value: i32, degree_type: DegreeType) -> Result<Self> {
        let (min, max) = match degree_type {
            DegreeType::PhD => (0, 0),
            DegreeType::Bachelor => (90, 240),
            DegreeType::Master => (60, 120),
        };

        if min == max {
            if value != min {
                return Err(MmsError::InvalidDegreeEcts {
                    ects: value,
                    degree_type: degree_type.to_string(),
                    expected_range: min.to_string(),
                });
            }
        } else if !(min..=max).contains(&value) {
            return Err(MmsError::InvalidDegreeEcts {
                ects: value,
                degree_type: degree_type.to_string(),
                expected_range: format!("{}-{}", min, max),
            });
        }

        Ok(DegreeEcts(value))
    }

    /// Get the inner value
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl Deref for DegreeEcts {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for DegreeEcts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe area ECTS value (must be positive)
///
/// Unlike course ECTS (1-30), area ECTS can be larger
/// (e.g., "Core CS" might require 60 ECTS across multiple courses)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AreaEcts(i32);

impl AreaEcts {
    /// Create a new AreaEcts value
    ///
    /// # Errors
    /// Returns `InvalidAreaEcts` if value is not positive
    pub fn new(value: i32) -> Result<Self> {
        if value <= 0 {
            return Err(MmsError::InvalidAreaEcts(value));
        }
        Ok(AreaEcts(value))
    }

    /// Get the inner value
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl Deref for AreaEcts {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for AreaEcts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degree_type_from_str() {
        assert_eq!(DegreeType::from_str("bachelor"), Ok(DegreeType::Bachelor));
        assert_eq!(DegreeType::from_str("Bachelor"), Ok(DegreeType::Bachelor));
        assert_eq!(DegreeType::from_str("b"), Ok(DegreeType::Bachelor));
        assert_eq!(DegreeType::from_str("bsc"), Ok(DegreeType::Bachelor));

        assert_eq!(DegreeType::from_str("master"), Ok(DegreeType::Master));
        assert_eq!(DegreeType::from_str("m"), Ok(DegreeType::Master));
        assert_eq!(DegreeType::from_str("msc"), Ok(DegreeType::Master));

        assert_eq!(DegreeType::from_str("phd"), Ok(DegreeType::PhD));
        assert_eq!(DegreeType::from_str("PhD"), Ok(DegreeType::PhD));
        assert_eq!(DegreeType::from_str("doctorate"), Ok(DegreeType::PhD));

        assert_eq!(DegreeType::from_str("invalid"), Err(()));
    }

    #[test]
    fn test_degree_type_display() {
        assert_eq!(DegreeType::Bachelor.to_string(), "bachelor");
        assert_eq!(DegreeType::Master.to_string(), "master");
        assert_eq!(DegreeType::PhD.to_string(), "phd");
    }

    #[test]
    fn test_serialize_deserialize() {
        let degree_type = DegreeType::Bachelor;
        let json = serde_json::to_string(&degree_type).unwrap();
        assert_eq!(json, "\"bachelor\"");

        let deserialized: DegreeType = serde_json::from_str("\"bachelor\"").unwrap();
        assert_eq!(deserialized, DegreeType::Bachelor);
    }

    #[test]
    fn test_degree_ects_phd() {
        assert!(DegreeEcts::new(0, DegreeType::PhD).is_ok());
        assert!(DegreeEcts::new(60, DegreeType::PhD).is_err());
    }

    #[test]
    fn test_degree_ects_bachelor() {
        assert!(DegreeEcts::new(180, DegreeType::Bachelor).is_ok());
        assert!(DegreeEcts::new(50, DegreeType::Bachelor).is_err());
        assert!(DegreeEcts::new(300, DegreeType::Bachelor).is_err());
    }

    #[test]
    fn test_degree_ects_master() {
        assert!(DegreeEcts::new(90, DegreeType::Master).is_ok());
        assert!(DegreeEcts::new(30, DegreeType::Master).is_err());
        assert!(DegreeEcts::new(150, DegreeType::Master).is_err());
    }

    #[test]
    fn test_area_ects() {
        assert!(AreaEcts::new(30).is_ok());
        assert!(AreaEcts::new(60).is_ok());
        assert!(AreaEcts::new(0).is_err());
        assert!(AreaEcts::new(-10).is_err());
    }
}
