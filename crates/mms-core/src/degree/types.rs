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

impl DegreeType {
    /// Parse from a string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bachelor" | "b" | "ba" | "bsc" => Some(DegreeType::Bachelor),
            "master" | "m" | "ma" | "msc" => Some(DegreeType::Master),
            "phd" | "doctorate" | "dr" => Some(DegreeType::PhD),
            _ => None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_degree_type_from_str() {
        assert_eq!(DegreeType::from_str("bachelor"), Some(DegreeType::Bachelor));
        assert_eq!(DegreeType::from_str("Bachelor"), Some(DegreeType::Bachelor));
        assert_eq!(DegreeType::from_str("b"), Some(DegreeType::Bachelor));
        assert_eq!(DegreeType::from_str("bsc"), Some(DegreeType::Bachelor));

        assert_eq!(DegreeType::from_str("master"), Some(DegreeType::Master));
        assert_eq!(DegreeType::from_str("m"), Some(DegreeType::Master));
        assert_eq!(DegreeType::from_str("msc"), Some(DegreeType::Master));

        assert_eq!(DegreeType::from_str("phd"), Some(DegreeType::PhD));
        assert_eq!(DegreeType::from_str("PhD"), Some(DegreeType::PhD));
        assert_eq!(DegreeType::from_str("doctorate"), Some(DegreeType::PhD));

        assert_eq!(DegreeType::from_str("invalid"), None);
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
}
