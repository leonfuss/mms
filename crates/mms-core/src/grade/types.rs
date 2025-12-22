use serde::{Deserialize, Serialize};

/// Grading scheme used for a course
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GradingScheme {
    /// German grading (1.0 - 5.0, lower is better)
    German,
    /// ECTS grading (A, B, C, D, E, F)
    ECTS,
    /// US GPA (0.0 - 4.0, higher is better)
    US,
    /// Percentage (0 - 100)
    Percentage,
    /// Pass/Fail
    PassFail,
}

impl GradingScheme {
    /// Parse from a string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "german" | "de" | "ger" => Some(GradingScheme::German),
            "ects" | "eu" | "european" => Some(GradingScheme::ECTS),
            "us" | "gpa" | "american" => Some(GradingScheme::US),
            "percentage" | "percent" | "%" => Some(GradingScheme::Percentage),
            "passfail" | "pass/fail" | "pf" | "passed" => Some(GradingScheme::PassFail),
            _ => None,
        }
    }

    /// Check if a grade is passing in this scheme
    pub fn is_passing(&self, grade: f64) -> bool {
        match self {
            GradingScheme::German => grade <= 4.0 && grade >= 1.0,
            GradingScheme::ECTS => grade <= 5.0 && grade >= 1.0, // A=1, B=2, C=3, D=4, E=5, F=6
            GradingScheme::US => grade >= 2.0, // Typically D (2.0) is minimum passing
            GradingScheme::Percentage => grade >= 50.0,
            GradingScheme::PassFail => grade >= 1.0, // 1 = pass, 0 = fail
        }
    }

    /// Validate if a grade value is valid for this scheme
    pub fn is_valid_grade(&self, grade: f64) -> bool {
        match self {
            GradingScheme::German => grade >= 1.0 && grade <= 5.0,
            GradingScheme::ECTS => grade >= 1.0 && grade <= 6.0,
            GradingScheme::US => grade >= 0.0 && grade <= 4.0,
            GradingScheme::Percentage => grade >= 0.0 && grade <= 100.0,
            GradingScheme::PassFail => grade == 0.0 || grade == 1.0,
        }
    }
}

impl std::fmt::Display for GradingScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GradingScheme::German => write!(f, "german"),
            GradingScheme::ECTS => write!(f, "ects"),
            GradingScheme::US => write!(f, "us"),
            GradingScheme::Percentage => write!(f, "percentage"),
            GradingScheme::PassFail => write!(f, "passfail"),
        }
    }
}

/// ECTS letter grade
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ECTSGrade {
    A, // Excellent (90-100%)
    B, // Very Good (80-89%)
    C, // Good (70-79%)
    D, // Satisfactory (60-69%)
    E, // Sufficient (50-59%)
    F, // Fail (<50%)
}

impl ECTSGrade {
    /// Convert ECTS letter to numeric value (A=1, B=2, ..., F=6)
    pub fn to_numeric(&self) -> f64 {
        match self {
            ECTSGrade::A => 1.0,
            ECTSGrade::B => 2.0,
            ECTSGrade::C => 3.0,
            ECTSGrade::D => 4.0,
            ECTSGrade::E => 5.0,
            ECTSGrade::F => 6.0,
        }
    }

    /// Convert numeric value to ECTS letter
    pub fn from_numeric(grade: f64) -> Option<Self> {
        match grade {
            g if g >= 1.0 && g < 1.5 => Some(ECTSGrade::A),
            g if g >= 1.5 && g < 2.5 => Some(ECTSGrade::B),
            g if g >= 2.5 && g < 3.5 => Some(ECTSGrade::C),
            g if g >= 3.5 && g < 4.5 => Some(ECTSGrade::D),
            g if g >= 4.5 && g < 5.5 => Some(ECTSGrade::E),
            g if g >= 5.5 => Some(ECTSGrade::F),
            _ => None,
        }
    }

    /// Convert from percentage
    pub fn from_percentage(percentage: f64) -> Self {
        match percentage {
            p if p >= 90.0 => ECTSGrade::A,
            p if p >= 80.0 => ECTSGrade::B,
            p if p >= 70.0 => ECTSGrade::C,
            p if p >= 60.0 => ECTSGrade::D,
            p if p >= 50.0 => ECTSGrade::E,
            _ => ECTSGrade::F,
        }
    }
}

impl std::fmt::Display for ECTSGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ECTSGrade::A => write!(f, "A"),
            ECTSGrade::B => write!(f, "B"),
            ECTSGrade::C => write!(f, "C"),
            ECTSGrade::D => write!(f, "D"),
            ECTSGrade::E => write!(f, "E"),
            ECTSGrade::F => write!(f, "F"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grading_scheme_from_str() {
        assert_eq!(GradingScheme::from_str("german"), Some(GradingScheme::German));
        assert_eq!(GradingScheme::from_str("German"), Some(GradingScheme::German));
        assert_eq!(GradingScheme::from_str("de"), Some(GradingScheme::German));

        assert_eq!(GradingScheme::from_str("ects"), Some(GradingScheme::ECTS));
        assert_eq!(GradingScheme::from_str("us"), Some(GradingScheme::US));
        assert_eq!(GradingScheme::from_str("gpa"), Some(GradingScheme::US));
        assert_eq!(GradingScheme::from_str("percentage"), Some(GradingScheme::Percentage));
        assert_eq!(GradingScheme::from_str("passfail"), Some(GradingScheme::PassFail));

        assert_eq!(GradingScheme::from_str("invalid"), None);
    }

    #[test]
    fn test_grading_scheme_is_passing() {
        assert!(GradingScheme::German.is_passing(1.0));
        assert!(GradingScheme::German.is_passing(2.7));
        assert!(GradingScheme::German.is_passing(4.0));
        assert!(!GradingScheme::German.is_passing(4.5));
        assert!(!GradingScheme::German.is_passing(5.0));

        assert!(GradingScheme::US.is_passing(3.5));
        assert!(GradingScheme::US.is_passing(2.0));
        assert!(!GradingScheme::US.is_passing(1.5));

        assert!(GradingScheme::Percentage.is_passing(75.0));
        assert!(GradingScheme::Percentage.is_passing(50.0));
        assert!(!GradingScheme::Percentage.is_passing(49.9));

        assert!(GradingScheme::PassFail.is_passing(1.0));
        assert!(!GradingScheme::PassFail.is_passing(0.0));
    }

    #[test]
    fn test_grading_scheme_is_valid() {
        assert!(GradingScheme::German.is_valid_grade(1.0));
        assert!(GradingScheme::German.is_valid_grade(2.7));
        assert!(GradingScheme::German.is_valid_grade(5.0));
        assert!(!GradingScheme::German.is_valid_grade(0.5));
        assert!(!GradingScheme::German.is_valid_grade(5.5));

        assert!(GradingScheme::US.is_valid_grade(0.0));
        assert!(GradingScheme::US.is_valid_grade(4.0));
        assert!(!GradingScheme::US.is_valid_grade(4.5));

        assert!(GradingScheme::Percentage.is_valid_grade(0.0));
        assert!(GradingScheme::Percentage.is_valid_grade(100.0));
        assert!(!GradingScheme::Percentage.is_valid_grade(101.0));

        assert!(GradingScheme::PassFail.is_valid_grade(0.0));
        assert!(GradingScheme::PassFail.is_valid_grade(1.0));
        assert!(!GradingScheme::PassFail.is_valid_grade(0.5));
    }

    #[test]
    fn test_ects_grade_conversions() {
        assert_eq!(ECTSGrade::A.to_numeric(), 1.0);
        assert_eq!(ECTSGrade::F.to_numeric(), 6.0);

        assert_eq!(ECTSGrade::from_numeric(1.0), Some(ECTSGrade::A));
        assert_eq!(ECTSGrade::from_numeric(3.0), Some(ECTSGrade::C));
        assert_eq!(ECTSGrade::from_numeric(6.0), Some(ECTSGrade::F));

        assert_eq!(ECTSGrade::from_percentage(95.0), ECTSGrade::A);
        assert_eq!(ECTSGrade::from_percentage(75.0), ECTSGrade::C);
        assert_eq!(ECTSGrade::from_percentage(55.0), ECTSGrade::E);
        assert_eq!(ECTSGrade::from_percentage(40.0), ECTSGrade::F);
    }

    #[test]
    fn test_grading_scheme_display() {
        assert_eq!(GradingScheme::German.to_string(), "german");
        assert_eq!(GradingScheme::ECTS.to_string(), "ects");
        assert_eq!(GradingScheme::US.to_string(), "us");
    }
}
