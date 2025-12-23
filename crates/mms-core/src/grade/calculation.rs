use super::conversion::calculate_weighted_average;
use super::types::GradingScheme;
use crate::error::Result;
use sea_orm::{DatabaseConnection, FromQueryResult};
use std::str::FromStr;

/// GPA calculation result
#[derive(Debug, Clone)]
pub struct GPAInfo {
    pub gpa: f64,
    pub total_courses: usize,
    pub total_ects: i32,
    pub grading_scheme: GradingScheme,
}

/// Detailed GPA breakdown by semester/degree area
#[derive(Debug, Clone)]
pub struct DetailedGPAInfo {
    pub overall_gpa: f64,
    pub total_courses: usize,
    pub total_ects: i32,
    pub grading_scheme: GradingScheme,
    pub per_semester: Vec<SemesterGPA>,
    pub per_degree_area: Vec<DegreeAreaGPA>,
}

#[derive(Debug, Clone)]
pub struct SemesterGPA {
    pub semester_id: i64,
    pub semester_code: String,
    pub gpa: f64,
    pub courses: usize,
    pub ects: i32,
}

#[derive(Debug, Clone)]
pub struct DegreeAreaGPA {
    pub area_id: i64,
    pub area_name: String,
    pub gpa: f64,
    pub courses: usize,
    pub ects: i32,
}

/// Internal struct for query results
#[derive(Debug, FromQueryResult)]
struct GradeWithEcts {
    grade: f64,
    grading_scheme: String,
    ects: i32,
}

/// Helper to calculate GPA from query results using functional patterns
fn calculate_gpa_from_results(results: Vec<GradeWithEcts>, scheme: GradingScheme) -> GPAInfo {
    if results.is_empty() {
        return GPAInfo {
            gpa: 0.0,
            total_courses: 0,
            total_ects: 0,
            grading_scheme: scheme,
        };
    }

    let (grade_ects_pairs, total_ects_vec): (Vec<(f64, f64)>, Vec<i32>) = results
        .into_iter()
        .map(|result| {
            let grade_value = if result.grading_scheme == scheme.to_string() {
                result.grade
            } else {
                let from_scheme = GradingScheme::from_str(&result.grading_scheme)
                    .unwrap_or(GradingScheme::German);
                super::conversion::convert_grade(result.grade, from_scheme, scheme)
                    .unwrap_or(result.grade)
            };
            ((grade_value, result.ects as f64), result.ects)
        })
        .unzip();

    let total_ects = total_ects_vec.iter().sum();
    let gpa = calculate_weighted_average(&grade_ects_pairs).unwrap_or(0.0);

    GPAInfo {
        gpa,
        total_courses: grade_ects_pairs.len(),
        total_ects,
        grading_scheme: scheme,
    }
}

/// Calculate overall GPA across all courses with final grades
///
/// This function:
/// 1. Fetches all courses with final grades
/// 2. Weights by ECTS credits
/// 3. Returns weighted average GPA
///
/// # Arguments
/// * `db` - Database connection
/// * `scheme` - Target grading scheme for the GPA
/// * `include_non_gpa`
///   - If `true`, includes grades from degree areas that don't count towards GPA.
///   - If `false` (recommended), only includes grades from GPA-counting areas.
pub async fn calculate_overall_gpa(
    db: &DatabaseConnection,
    scheme: GradingScheme,
    include_non_gpa: bool,
) -> Result<GPAInfo> {
    let query = if include_non_gpa {
        // Include all grades regardless of degree area settings
        r#"
            SELECT DISTINCT g.grade, g.grading_scheme, c.ects
            FROM grades g
            INNER JOIN courses c ON g.course_id = c.id
            WHERE g.is_final = 1
              AND g.passed = 1
        "#
    } else {
        // Only include grades from courses in GPA-counting degree areas
        r#"
            SELECT DISTINCT g.grade, g.grading_scheme, c.ects
            FROM grades g
            INNER JOIN courses c ON g.course_id = c.id
            INNER JOIN course_degree_mappings cdm ON c.id = cdm.course_id
            INNER JOIN degree_areas da ON cdm.degree_area_id = da.id
            WHERE g.is_final = 1
              AND g.passed = 1
              AND da.counts_towards_gpa = 1
        "#
    };

    let results = GradeWithEcts::find_by_statement(sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        query,
        vec![],
    ))
    .all(db)
    .await?;

    Ok(calculate_gpa_from_results(results, scheme))
}

/// Calculate GPA for a specific semester
///
/// # Arguments
/// * `db` - Database connection
/// * `semester_id` - ID of the semester
/// * `scheme` - Target grading scheme for the GPA
/// * `include_non_gpa`
///   - If `true`, includes grades from degree areas that don't count towards GPA.
///   - If `false` (recommended), only includes grades from GPA-counting areas.
pub async fn calculate_semester_gpa(
    db: &DatabaseConnection,
    semester_id: i64,
    scheme: GradingScheme,
    include_non_gpa: bool,
) -> Result<GPAInfo> {
    let query = if include_non_gpa {
        // Include all grades regardless of degree area settings
        r#"
            SELECT DISTINCT g.grade, g.grading_scheme, c.ects
            FROM grades g
            INNER JOIN courses c ON g.course_id = c.id
            WHERE g.is_final = 1
              AND g.passed = 1
              AND c.semester_id = ?
        "#
    } else {
        // Only include grades from courses in GPA-counting degree areas
        r#"
            SELECT DISTINCT g.grade, g.grading_scheme, c.ects
            FROM grades g
            INNER JOIN courses c ON g.course_id = c.id
            INNER JOIN course_degree_mappings cdm ON c.id = cdm.course_id
            INNER JOIN degree_areas da ON cdm.degree_area_id = da.id
            WHERE g.is_final = 1
              AND g.passed = 1
              AND c.semester_id = ?
              AND da.counts_towards_gpa = 1
        "#
    };

    let results = GradeWithEcts::find_by_statement(sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        query,
        vec![semester_id.into()],
    ))
    .all(db)
    .await?;

    Ok(calculate_gpa_from_results(results, scheme))
}

/// Calculate GPA for a specific degree
pub async fn calculate_degree_gpa(
    db: &DatabaseConnection,
    degree_id: i64,
    scheme: GradingScheme,
) -> Result<GPAInfo> {
    // Get all courses mapped to this degree's areas with final grades
    let query = r#"
        SELECT g.grade, g.grading_scheme, c.ects
        FROM grades g
        INNER JOIN courses c ON g.course_id = c.id
        INNER JOIN course_degree_mappings cdm ON c.id = cdm.course_id
        INNER JOIN degree_areas da ON cdm.degree_area_id = da.id
        WHERE da.degree_id = ?
          AND g.is_final = 1
          AND g.passed = 1
          AND da.counts_towards_gpa = 1
    "#;

    let results = GradeWithEcts::find_by_statement(sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        query,
        vec![degree_id.into()],
    ))
    .all(db)
    .await?;

    Ok(calculate_gpa_from_results(results, scheme))
}

/// Calculate GPA for a specific degree area
pub async fn calculate_degree_area_gpa(
    db: &DatabaseConnection,
    degree_area_id: i64,
    scheme: GradingScheme,
) -> Result<GPAInfo> {
    let query = r#"
        SELECT g.grade, g.grading_scheme, c.ects
        FROM grades g
        INNER JOIN courses c ON g.course_id = c.id
        INNER JOIN course_degree_mappings cdm ON c.id = cdm.course_id
        WHERE cdm.degree_area_id = ?
          AND g.is_final = 1
          AND g.passed = 1
    "#;

    let results = GradeWithEcts::find_by_statement(sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        query,
        vec![degree_area_id.into()],
    ))
    .all(db)
    .await?;

    Ok(calculate_gpa_from_results(results, scheme))
}

/// Get comprehensive GPA statistics
///
/// # Arguments
/// * `db` - Database connection
/// * `scheme` - Target grading scheme for the GPA
/// * `include_non_gpa`
///    - If `true`, includes grades from degree areas that don't count towards GPA.
///    - If `false` (recommended), only includes grades from GPA-counting areas.
pub async fn get_detailed_gpa(
    db: &DatabaseConnection,
    scheme: GradingScheme,
    include_non_gpa: bool,
) -> Result<DetailedGPAInfo> {
    let overall = calculate_overall_gpa(db, scheme, include_non_gpa).await?;

    // TODO: Implement per-semester and per-degree-area breakdowns
    // This would require joining with semesters and degree_areas tables

    Ok(DetailedGPAInfo {
        overall_gpa: overall.gpa,
        total_courses: overall.total_courses,
        total_ects: overall.total_ects,
        grading_scheme: scheme,
        per_semester: Vec::new(),    // TODO
        per_degree_area: Vec::new(), // TODO
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are unit tests for the calculation logic.
    // Integration tests with actual database would go in tests/ directory.

    #[test]
    fn test_gpa_info_creation() {
        let info = GPAInfo {
            gpa: 2.3,
            total_courses: 10,
            total_ects: 60,
            grading_scheme: GradingScheme::German,
        };

        assert_eq!(info.gpa, 2.3);
        assert_eq!(info.total_courses, 10);
        assert_eq!(info.total_ects, 60);
    }

    #[test]
    fn test_calculate_gpa_from_results() {
        // Case 1: Empty results
        let info = calculate_gpa_from_results(vec![], GradingScheme::German);
        assert_eq!(info.gpa, 0.0);
        assert_eq!(info.total_courses, 0);
        assert_eq!(info.total_ects, 0);

        // Case 2: Single result, same scheme
        let results = vec![GradeWithEcts {
            grade: 1.0,
            grading_scheme: "german".to_string(),
            ects: 5,
        }];
        let info = calculate_gpa_from_results(results, GradingScheme::German);
        assert_eq!(info.gpa, 1.0);
        assert_eq!(info.total_courses, 1);
        assert_eq!(info.total_ects, 5);

        // Case 3: Multiple results, mixed schemes
        let results = vec![
            GradeWithEcts {
                grade: 1.0,
                grading_scheme: "german".to_string(),
                ects: 5,
            },
            GradeWithEcts {
                grade: 4.0, // A in US is 4.0
                grading_scheme: "us".to_string(),
                ects: 5,
            },
        ];
        // Convert US 4.0 to German -> approx 1.0
        // Weighted average of 1.0 (5 ECTS) and ~1.0 (5 ECTS) should be ~1.0
        let info = calculate_gpa_from_results(results, GradingScheme::German);
        assert!((info.gpa - 1.0).abs() < 0.1);
        assert_eq!(info.total_courses, 2);
        assert_eq!(info.total_ects, 10);
    }
}
