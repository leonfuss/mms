use crate::error::{MmsError, Result};
use std::fmt;
use std::ops::Deref;

/// Type-safe ECTS value (1-30)
///
/// Guarantees at compile time that ECTS values are within valid range.
///
/// # Example
/// ```
/// use mms_core::course::Ects;
///
/// let ects = Ects::new(6).unwrap();
/// assert_eq!(ects.value(), 6);
///
/// // Invalid values are rejected
/// assert!(Ects::new(0).is_err());
/// assert!(Ects::new(31).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ects(i32);

impl Ects {
    /// Create a new ECTS value
    ///
    /// # Errors
    /// Returns `InvalidEcts` if value is not in range 1-30
    pub fn new(value: i32) -> Result<Self> {
        if !(1..=30).contains(&value) {
            return Err(MmsError::InvalidEcts { value });
        }
        Ok(Ects(value))
    }

    /// Get the inner value
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl Deref for Ects {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Ects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe course code (directory-safe string)
///
/// Guarantees at compile time that course codes are valid directory names.
/// Valid characters: alphanumeric, hyphen, underscore. No leading dots.
///
/// # Example
/// ```
/// use mms_core::course::CourseCode;
///
/// let code = CourseCode::new("cs101".to_string()).unwrap();
/// assert_eq!(code.as_str(), "cs101");
///
/// // Invalid codes are rejected
/// assert!(CourseCode::new("CS 101".to_string()).is_err()); // space
/// assert!(CourseCode::new(".hidden".to_string()).is_err()); // leading dot
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CourseCode(String);

impl CourseCode {
    /// Create a new course code
    ///
    /// # Errors
    /// Returns `InvalidCourseCode` if the code contains invalid characters
    pub fn new(code: String) -> Result<Self> {
        if code.is_empty() {
            return Err(MmsError::InvalidCourseCode {
                code,
                reason: "empty".to_string(),
            });
        }
        if code.starts_with('.') {
            return Err(MmsError::InvalidCourseCode {
                code,
                reason: "starts with dot".to_string(),
            });
        }
        if !code
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(MmsError::InvalidCourseCode {
                code,
                reason: "contains invalid characters".to_string(),
            });
        }
        Ok(CourseCode(code))
    }

    /// Get the code as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the inner String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl Deref for CourseCode {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for CourseCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CourseCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Ects Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_ects_valid() {
        assert!(Ects::new(1).is_ok());
        assert!(Ects::new(6).is_ok());
        assert!(Ects::new(15).is_ok());
        assert!(Ects::new(30).is_ok());
    }

    #[test]
    fn test_ects_invalid() {
        assert!(Ects::new(0).is_err());
        assert!(Ects::new(-1).is_err());
        assert!(Ects::new(31).is_err());
        assert!(Ects::new(100).is_err());
    }

    #[test]
    fn test_ects_value() {
        let ects = Ects::new(6).unwrap();
        assert_eq!(ects.value(), 6);
        assert_eq!(*ects, 6); // Test Deref
    }

    #[test]
    fn test_ects_display() {
        let ects = Ects::new(12).unwrap();
        assert_eq!(format!("{}", ects), "12");
    }

    // ------------------------------------------------------------------------
    // CourseCode Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_course_code_valid() {
        assert!(CourseCode::new("cs101".to_string()).is_ok());
        assert!(CourseCode::new("math-201".to_string()).is_ok());
        assert!(CourseCode::new("phys_1".to_string()).is_ok());
        assert!(CourseCode::new("BIO2".to_string()).is_ok());
    }

    #[test]
    fn test_course_code_invalid() {
        assert!(CourseCode::new("".to_string()).is_err()); // empty
        assert!(CourseCode::new(".hidden".to_string()).is_err()); // leading dot
        assert!(CourseCode::new("CS 101".to_string()).is_err()); // space
        assert!(CourseCode::new("foo/bar".to_string()).is_err()); // slash
        assert!(CourseCode::new("test@example".to_string()).is_err()); // special char
    }

    #[test]
    fn test_course_code_as_str() {
        let code = CourseCode::new("cs101".to_string()).unwrap();
        assert_eq!(code.as_str(), "cs101");
        assert_eq!(&*code, "cs101"); // Test Deref
    }

    #[test]
    fn test_course_code_display() {
        let code = CourseCode::new("math201".to_string()).unwrap();
        assert_eq!(format!("{}", code), "math201");
    }

    #[test]
    fn test_course_code_into_string() {
        let code = CourseCode::new("phys101".to_string()).unwrap();
        let s = code.into_string();
        assert_eq!(s, "phys101");
    }
}
