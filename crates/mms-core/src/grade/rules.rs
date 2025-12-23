use std::str::FromStr;

/// A rule for matching a grade value.
///
/// Supported formats:
/// - Point: "1.0", "2.3" (Exact match)
/// - Range: "1.0-1.5" (Inclusive start, Inclusive end)
/// - Threshold: "4.0+" (Greater than or equal) or "4.0-" (Less than or equal - though "-" is ambiguous with range, using suffixes like "+" or prefix "<"/"<=" might be better, but sticking to simple string parsing)
///
/// Note: German grades are "lower is better", so "1.0-1.5" matches the best grades.
#[derive(Debug, PartialEq, Clone)]
pub enum GradeRule {
    /// Exact match (e.g., "28")
    Point(f64),
    /// Range of values (e.g., "1.0-1.5"). Inclusive on both ends for this implementation.
    Range { min: f64, max: f64 },
    /// Open-ended threshold.
    /// "30+" -> value >= 30
    /// "1.0-" -> value <= 1.0 (Not strictly required by prompt but useful)
    Threshold { value: f64, distinct: ThresholdType },
}

#[derive(Debug, PartialEq, Clone)]
pub enum ThresholdType {
    AtLeast, // value >= threshold ("+")
    AtMost,  // value <= threshold ("-")
}

impl FromStr for GradeRule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err("Empty rule string".to_string());
        }

        // Check for Range "min-max"
        // Note: Negative numbers might confuse this simple split. Assuming positive grades.
        if let Some((start, end)) = s.split_once('-') {
            // Check if it is a threshold "1.0-"
            if end.is_empty() {
                 let val = start.parse::<f64>().map_err(|_| format!("Invalid number in threshold: {}", start))?;
                 return Ok(GradeRule::Threshold { value: val, distinct: ThresholdType::AtMost });
            }
            // It's a range "1.0-1.5"
            let min = start.parse::<f64>().map_err(|_| format!("Invalid min in range: {}", start))?;
            let max = end.parse::<f64>().map_err(|_| format!("Invalid max in range: {}", end))?;

            // Precondition: Range must be valid
            if min > max {
                return Err(format!("Invalid range: min {} > max {}", min, max));
            }

            return Ok(GradeRule::Range { min, max });
        }

        // Check for Threshold "30+"
        if let Some(val_str) = s.strip_suffix('+') {
            let val = val_str.parse::<f64>().map_err(|_| format!("Invalid number in threshold: {}", val_str))?;
            return Ok(GradeRule::Threshold { value: val, distinct: ThresholdType::AtLeast });
        }

        // Exact Point
        let val = s.parse::<f64>().map_err(|_| format!("Invalid point: {}", s))?;
        Ok(GradeRule::Point(val))
    }
}

impl GradeRule {
    /// Check if the given value matches the rule.
    pub fn matches(&self, value: f64) -> bool {
        match self {
            GradeRule::Point(p) => (value - p).abs() < f64::EPSILON,
            GradeRule::Range { min, max } => value >= *min && value <= *max,
            GradeRule::Threshold { value: t, distinct } => match distinct {
                ThresholdType::AtLeast => value >= *t,
                ThresholdType::AtMost => value <= *t,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_point() {
        assert_eq!("1.0".parse::<GradeRule>(), Ok(GradeRule::Point(1.0)));
        assert_eq!("2.3".parse::<GradeRule>(), Ok(GradeRule::Point(2.3)));
    }

    #[test]
    fn test_parse_range() {
        assert_eq!("1.0-1.5".parse::<GradeRule>(), Ok(GradeRule::Range { min: 1.0, max: 1.5 }));
        assert_eq!("1.5-2.5".parse::<GradeRule>(), Ok(GradeRule::Range { min: 1.5, max: 2.5 }));

        // Error cases
        assert!("2.5-1.5".parse::<GradeRule>().is_err());
        assert!("abc-1.5".parse::<GradeRule>().is_err());
    }

    #[test]
    fn test_parse_threshold() {
        assert_eq!("30+".parse::<GradeRule>(), Ok(GradeRule::Threshold { value: 30.0, distinct: ThresholdType::AtLeast }));
        assert_eq!("1.0-".parse::<GradeRule>(), Ok(GradeRule::Threshold { value: 1.0, distinct: ThresholdType::AtMost }));
    }

    #[test]
    fn test_matches() {
        let range = GradeRule::Range { min: 1.0, max: 1.5 };
        assert!(range.matches(1.0));
        assert!(range.matches(1.3));
        assert!(range.matches(1.5));
        assert!(!range.matches(1.6));

        let point = GradeRule::Point(2.0);
        assert!(point.matches(2.0));
        assert!(!point.matches(2.00001));

        let threshold = GradeRule::Threshold { value: 4.0, distinct: ThresholdType::AtLeast };
        assert!(threshold.matches(4.0));
        assert!(threshold.matches(5.0));
        assert!(!threshold.matches(3.9));
    }
}
