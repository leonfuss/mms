use super::types::{ECTSGrade, GradingScheme};

/// Convert a grade from one scheme to another
pub fn convert_grade(
    grade: f64,
    from_scheme: GradingScheme,
    to_scheme: GradingScheme,
) -> Option<f64> {
    if from_scheme == to_scheme {
        return Some(grade);
    }

    // First convert to percentage (universal intermediate)
    let percentage = to_percentage(grade, from_scheme)?;

    // Then convert from percentage to target scheme
    from_percentage(percentage, to_scheme)
}

/// Convert any grade to percentage (0-100)
fn to_percentage(grade: f64, scheme: GradingScheme) -> Option<f64> {
    if !scheme.is_valid_grade(grade) {
        return None;
    }

    match scheme {
        GradingScheme::German => {
            // German: 1.0 (best) -> 100%, 4.0 (worst passing) -> 50%
            // Linear mapping: percentage = 100 - (grade - 1.0) * 50 / 3.0
            let percentage = 100.0 - ((grade - 1.0) * 50.0 / 3.0);
            Some(percentage.clamp(0.0, 100.0))
        }
        GradingScheme::ECTS => {
            // Convert ECTS numeric to letter, then to percentage
            let ects_letter = ECTSGrade::from_numeric(grade)?;
            Some(match ects_letter {
                ECTSGrade::A => 95.0,
                ECTSGrade::B => 85.0,
                ECTSGrade::C => 75.0,
                ECTSGrade::D => 65.0,
                ECTSGrade::E => 55.0,
                ECTSGrade::F => 40.0,
            })
        }
        GradingScheme::US => {
            // US GPA: 4.0 -> 100%, 0.0 -> 0%
            Some((grade / 4.0) * 100.0)
        }
        GradingScheme::Percentage => Some(grade),
        GradingScheme::PassFail => Some(if grade >= 1.0 { 100.0 } else { 0.0 }),
    }
}

/// Convert percentage to any grading scheme
fn from_percentage(percentage: f64, scheme: GradingScheme) -> Option<f64> {
    if !(0.0..=100.0).contains(&percentage) {
        return None;
    }

    match scheme {
        GradingScheme::German => {
            // Reverse of to_percentage: grade = 1.0 + (100 - percentage) * 3.0 / 50.0
            let grade = 1.0 + ((100.0 - percentage) * 3.0 / 50.0);
            Some(grade.clamp(1.0, 5.0))
        }
        GradingScheme::ECTS => {
            let ects_letter = ECTSGrade::from_percentage(percentage);
            Some(ects_letter.to_numeric())
        }
        GradingScheme::US => Some((percentage / 100.0) * 4.0),
        GradingScheme::Percentage => Some(percentage),
        GradingScheme::PassFail => Some(if percentage >= 50.0 { 1.0 } else { 0.0 }),
    }
}

/// Convert German grade to ECTS letter grade
pub fn german_to_ects(german_grade: f64) -> Option<ECTSGrade> {
    if !GradingScheme::German.is_valid_grade(german_grade) {
        return None;
    }

    match german_grade {
        g if (1.0..=1.5).contains(&g) => Some(ECTSGrade::A), // Excellent
        g if g > 1.5 && g <= 2.5 => Some(ECTSGrade::B),      // Very Good
        g if g > 2.5 && g <= 3.5 => Some(ECTSGrade::C),      // Good
        g if g > 3.5 && g <= 4.0 => Some(ECTSGrade::D),      // Satisfactory
        g if g > 4.0 && g <= 5.0 => Some(ECTSGrade::F),      // Fail
        _ => None,
    }
}

/// Convert US GPA to German grade
pub fn us_to_german(us_gpa: f64) -> Option<f64> {
    if !GradingScheme::US.is_valid_grade(us_gpa) {
        return None;
    }

    // Approximate conversion:
    // 4.0 (A) -> 1.0
    // 3.0 (B) -> 2.0
    // 2.0 (C) -> 3.0
    // 1.0 (D) -> 4.0
    // 0.0 (F) -> 5.0
    let german = 5.0 - us_gpa;
    Some(german.clamp(1.0, 5.0))
}

/// Convert German grade to US GPA
pub fn german_to_us(german_grade: f64) -> Option<f64> {
    if !GradingScheme::German.is_valid_grade(german_grade) {
        return None;
    }

    let us_gpa = 5.0 - german_grade;
    Some(us_gpa.clamp(0.0, 4.0))
}

/// Calculate weighted average of grades
pub fn calculate_weighted_average(grades: &[(f64, f64)]) -> Option<f64> {
    if grades.is_empty() {
        return None;
    }

    let total_weight: f64 = grades.iter().map(|(_, weight)| weight).sum();
    if total_weight == 0.0 {
        return None;
    }

    let weighted_sum: f64 = grades.iter().map(|(grade, weight)| grade * weight).sum();
    Some(weighted_sum / total_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_german_to_percentage() {
        let p1 = to_percentage(1.0, GradingScheme::German).unwrap();
        assert!((p1 - 100.0).abs() < 0.1);

        let p2 = to_percentage(4.0, GradingScheme::German).unwrap();
        assert!((p2 - 50.0).abs() < 0.1);

        let p3 = to_percentage(2.5, GradingScheme::German).unwrap();
        assert!(p3 > 70.0 && p3 < 80.0);
    }

    #[test]
    fn test_percentage_to_german() {
        let g1 = from_percentage(100.0, GradingScheme::German).unwrap();
        assert!((g1 - 1.0).abs() < 0.1);

        let g2 = from_percentage(50.0, GradingScheme::German).unwrap();
        assert!((g2 - 4.0).abs() < 0.1);
    }

    #[test]
    fn test_convert_grade() {
        // German to US
        let us = convert_grade(1.0, GradingScheme::German, GradingScheme::US).unwrap();
        assert!(us > 3.5); // 1.0 German should be high US GPA

        // German to ECTS
        let ects = convert_grade(1.3, GradingScheme::German, GradingScheme::ECTS).unwrap();
        assert_eq!(ects, 1.0); // Should be A

        // Same scheme
        let same = convert_grade(2.7, GradingScheme::German, GradingScheme::German).unwrap();
        assert_eq!(same, 2.7);
    }

    #[test]
    fn test_german_to_ects() {
        assert_eq!(german_to_ects(1.0), Some(ECTSGrade::A));
        assert_eq!(german_to_ects(1.5), Some(ECTSGrade::A));
        assert_eq!(german_to_ects(2.0), Some(ECTSGrade::B));
        assert_eq!(german_to_ects(3.0), Some(ECTSGrade::C));
        assert_eq!(german_to_ects(4.0), Some(ECTSGrade::D));
        assert_eq!(german_to_ects(5.0), Some(ECTSGrade::F));

        // Invalid grade
        assert_eq!(german_to_ects(6.0), None);
    }

    #[test]
    fn test_us_german_conversion() {
        let german = us_to_german(4.0).unwrap();
        assert!((german - 1.0).abs() < 0.1);

        let us = german_to_us(1.0).unwrap();
        assert!((us - 4.0).abs() < 0.1);
    }

    #[test]
    fn test_weighted_average() {
        let grades = vec![(1.0, 0.4), (2.0, 0.6)];
        let avg = calculate_weighted_average(&grades).unwrap();
        assert!((avg - 1.6).abs() < 0.01);

        let grades2 = vec![(90.0, 1.0), (80.0, 2.0), (70.0, 1.0)];
        let avg2 = calculate_weighted_average(&grades2).unwrap();
        assert!((avg2 - 80.0).abs() < 0.01);

        // Empty grades
        assert_eq!(calculate_weighted_average(&[]), None);
    }

    #[test]
    fn test_conversion_roundtrip() {
        // German -> US -> German
        let original = 2.3;
        let us = convert_grade(original, GradingScheme::German, GradingScheme::US).unwrap();
        let back = convert_grade(us, GradingScheme::US, GradingScheme::German).unwrap();
        assert!((original - back).abs() < 0.2); // Allow small error

        // Percentage -> ECTS -> Percentage
        let original_pct = 85.0;
        let ects =
            convert_grade(original_pct, GradingScheme::Percentage, GradingScheme::ECTS).unwrap();
        let back_pct = convert_grade(ects, GradingScheme::ECTS, GradingScheme::Percentage).unwrap();
        assert!((original_pct - back_pct).abs() < 10.0); // ECTS has coarse granularity
    }
}
