use super::conversion::calculate_weighted_average;
use super::operations::{record_grade, GradeInfo};
use super::types::GradingScheme;
use crate::error::Result;
use sea_orm::DatabaseConnection;

/// Grade component definition for GradeBuilder
#[derive(Debug, Clone)]
pub struct ComponentDefinition {
    pub name: String,
    pub weight: f64,
    pub points_earned: Option<f64>,
    pub points_total: Option<f64>,
    pub grade: Option<f64>,
    pub is_bonus: bool,
    pub bonus_points: Option<f64>,
}

/// Builder for creating a grade with components
///
/// # Example
/// ```no_run
/// use mms_core::grade::{GradeBuilder, GradingScheme};
/// use sea_orm::DatabaseConnection;
///
/// # async fn example(db: &DatabaseConnection, course_id: i64) -> mms_core::error::Result<()> {
/// let grade = GradeBuilder::new(course_id, 1.7)
///     .with_scheme(GradingScheme::German)
///     .with_component("Midterm Exam", 0.4, Some(85.0), Some(100.0))
///     .with_component("Final Exam", 0.6, Some(90.0), Some(100.0))
///     .with_attempt(1)
///     .with_exam_date("2024-02-15")
///     .as_final(true)
///     .record(db)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct GradeBuilder {
    course_id: i64,
    grade: f64,
    grading_scheme: GradingScheme,
    original_grade: Option<f64>,
    original_scheme: Option<GradingScheme>,
    is_final: bool,
    attempt_number: i64,
    exam_date: Option<String>,
    components: Vec<ComponentDefinition>,
}

impl GradeBuilder {
    /// Create a new grade builder
    ///
    /// # Arguments
    /// * `course_id` - ID of the course this grade is for
    /// * `grade` - The final grade value
    pub fn new(course_id: i64, grade: f64) -> Self {
        Self {
            course_id,
            grade,
            grading_scheme: GradingScheme::German, // Default to German grading
            original_grade: None,
            original_scheme: None,
            is_final: false,
            attempt_number: 1,
            exam_date: None,
            components: Vec::new(),
        }
    }

    /// Set the grading scheme
    pub fn with_scheme(mut self, scheme: GradingScheme) -> Self {
        self.grading_scheme = scheme;
        self
    }

    /// Set original grade if this was converted from another scheme
    pub fn with_original(mut self, original_grade: f64, original_scheme: GradingScheme) -> Self {
        self.original_grade = Some(original_grade);
        self.original_scheme = Some(original_scheme);
        self
    }

    /// Set whether this is the final grade
    pub fn as_final(mut self, is_final: bool) -> Self {
        self.is_final = is_final;
        self
    }

    /// Set the attempt number (for retakes)
    pub fn with_attempt(mut self, attempt: i64) -> Self {
        self.attempt_number = attempt;
        self
    }

    /// Set the exam date
    pub fn with_exam_date<S: Into<String>>(mut self, date: S) -> Self {
        self.exam_date = Some(date.into());
        self
    }

    /// Add a grade component (exam, homework, project, etc.)
    ///
    /// # Arguments
    /// * `name` - Component name (e.g., "Midterm Exam")
    /// * `weight` - Weight in final grade (e.g., 0.4 for 40%)
    /// * `points_earned` - Points earned (optional)
    /// * `points_total` - Total points possible (optional)
    pub fn with_component<S: Into<String>>(
        mut self,
        name: S,
        weight: f64,
        points_earned: Option<f64>,
        points_total: Option<f64>,
    ) -> Self {
        self.components.push(ComponentDefinition {
            name: name.into(),
            weight,
            points_earned,
            points_total,
            grade: None,
            is_bonus: false,
            bonus_points: None,
        });
        self
    }

    /// Add a grade component with an explicit grade value
    pub fn with_graded_component<S: Into<String>>(
        mut self,
        name: S,
        weight: f64,
        grade: f64,
    ) -> Self {
        self.components.push(ComponentDefinition {
            name: name.into(),
            weight,
            points_earned: None,
            points_total: None,
            grade: Some(grade),
            is_bonus: false,
            bonus_points: None,
        });
        self
    }

    /// Add a bonus component
    pub fn with_bonus<S: Into<String>>(
        mut self,
        name: S,
        bonus_points: f64,
    ) -> Self {
        self.components.push(ComponentDefinition {
            name: name.into(),
            weight: 0.0,
            points_earned: None,
            points_total: None,
            grade: None,
            is_bonus: true,
            bonus_points: Some(bonus_points),
        });
        self
    }

    /// Calculate final grade from components if no grade was explicitly set
    pub fn calculate_from_components(mut self) -> Self {
        if self.components.is_empty() {
            return self;
        }

        // Collect components with grades
        let graded_components: Vec<(f64, f64)> = self
            .components
            .iter()
            .filter_map(|c| {
                // Try to use explicit grade first, then calculate from points
                c.grade
                    .map(|g| (g, c.weight))
                    .or_else(|| {
                        c.points_earned.zip(c.points_total).and_then(|(earned, total)| {
                            if total > 0.0 {
                                let percentage = (earned / total) * 100.0;
                                Some((percentage, c.weight))
                            } else {
                                None
                            }
                        })
                    })
            })
            .collect();

        if let Some(calculated_grade) = calculate_weighted_average(&graded_components) {
            self.grade = calculated_grade;
        }

        self
    }

    /// Record the grade in the database
    ///
    /// This will:
    /// 1. Create the grade entry
    /// 2. Create all component entries
    /// 3. Return the complete grade info
    pub async fn record(self, db: &DatabaseConnection) -> Result<GradeInfo> {
        record_grade(
            db,
            self.course_id,
            self.grade,
            self.grading_scheme,
            self.original_grade,
            self.original_scheme,
            self.is_final,
            self.attempt_number,
            self.exam_date,
            self.components,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let builder = GradeBuilder::new(1, 1.7);
        assert_eq!(builder.course_id, 1);
        assert_eq!(builder.grade, 1.7);
        assert_eq!(builder.grading_scheme, GradingScheme::German);
        assert_eq!(builder.is_final, false);
        assert_eq!(builder.attempt_number, 1);
    }

    #[test]
    fn test_builder_with_details() {
        let builder = GradeBuilder::new(1, 2.3)
            .with_scheme(GradingScheme::ECTS)
            .as_final(true)
            .with_attempt(2)
            .with_exam_date("2024-02-15");

        assert_eq!(builder.grading_scheme, GradingScheme::ECTS);
        assert_eq!(builder.is_final, true);
        assert_eq!(builder.attempt_number, 2);
        assert_eq!(builder.exam_date, Some("2024-02-15".to_string()));
    }

    #[test]
    fn test_builder_with_components() {
        let builder = GradeBuilder::new(1, 0.0)
            .with_component("Midterm", 0.4, Some(85.0), Some(100.0))
            .with_component("Final", 0.6, Some(90.0), Some(100.0));

        assert_eq!(builder.components.len(), 2);
        assert_eq!(builder.components[0].name, "Midterm");
        assert_eq!(builder.components[0].weight, 0.4);
        assert_eq!(builder.components[0].points_earned, Some(85.0));
    }

    #[test]
    fn test_builder_with_graded_components() {
        let builder = GradeBuilder::new(1, 0.0)
            .with_graded_component("Midterm", 0.4, 1.7)
            .with_graded_component("Final", 0.6, 2.0);

        assert_eq!(builder.components.len(), 2);
        assert_eq!(builder.components[0].grade, Some(1.7));
        assert_eq!(builder.components[1].grade, Some(2.0));
    }

    #[test]
    fn test_calculate_from_components() {
        let builder = GradeBuilder::new(1, 0.0)
            .with_graded_component("Midterm", 0.4, 1.0)
            .with_graded_component("Final", 0.6, 2.0)
            .calculate_from_components();

        // 1.0 * 0.4 + 2.0 * 0.6 = 0.4 + 1.2 = 1.6
        assert!((builder.grade - 1.6).abs() < 0.01);
    }

    #[test]
    fn test_calculate_from_points() {
        let builder = GradeBuilder::new(1, 0.0)
            .with_component("Midterm", 0.5, Some(85.0), Some(100.0))
            .with_component("Final", 0.5, Some(90.0), Some(100.0))
            .calculate_from_components();

        // (85/100) * 0.5 + (90/100) * 0.5 = 85 * 0.5 + 90 * 0.5 = 87.5
        assert!((builder.grade - 87.5).abs() < 0.01);
    }

    #[test]
    fn test_builder_with_bonus() {
        let builder = GradeBuilder::new(1, 1.7)
            .with_bonus("Extra Credit", 5.0);

        assert_eq!(builder.components.len(), 1);
        assert_eq!(builder.components[0].is_bonus, true);
        assert_eq!(builder.components[0].bonus_points, Some(5.0));
    }

    #[test]
    fn test_builder_with_original() {
        let builder = GradeBuilder::new(1, 1.3)
            .with_scheme(GradingScheme::German)
            .with_original(3.7, GradingScheme::US);

        assert_eq!(builder.original_grade, Some(3.7));
        assert_eq!(builder.original_scheme, Some(GradingScheme::US));
    }
}
