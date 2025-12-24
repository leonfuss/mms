/// Example demonstrating the grade management and GPA calculation API
///
/// This example shows how to:
/// 1. Record grades with different grading schemes
/// 2. Add grade components (exams, homework, projects)
/// 3. Convert between grading schemes
/// 4. Calculate GPAs (overall, by semester, by degree)
/// 5. Query and update grades
///
/// Run with: cargo run --example grade_example
use mms_core::config::Config;
use mms_core::course::CourseBuilder;
use mms_core::db::connection_seaorm;
use mms_core::degree::{DegreeBuilder, DegreeType, map_course_to_area};
use mms_core::grade::{
    GradeBuilder, GradingScheme, calculate_degree_gpa, calculate_overall_gpa,
    calculate_semester_gpa, convert_grade, german_to_ects, get_final_grade, get_grade_by_id,
};
use mms_core::semester::{SemesterBuilder, SemesterType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Grade Management & GPA Calculation API Example ===\n");

    // Load configuration and connect to database
    println!("1. Loading configuration and connecting to database...");
    let config = Config::load()?;
    let db = connection_seaorm::get_connection().await?;
    println!("   ✓ Connected!\n");

    // ================================================================================
    // SETUP: Create degree, semester, and courses
    // ================================================================================

    println!("2. Setting up degree, semester, and courses...");

    // Create a degree
    // Note: "Core CS" counts towards GPA (true), "Electives" does NOT (false)
    let degree = DegreeBuilder::new(DegreeType::Bachelor, "Computer Science", "TUM")
        .with_total_ects(180)
        .with_area("Core CS", 60, true) // Counts towards GPA
        .with_area("Electives", 30, false) // Does NOT count towards GPA (still need ECTS though)
        .create(&db)
        .await?;

    // Create a semester
    let semester = SemesterBuilder::new(SemesterType::Bachelor, 3)
        .with_start_date("2024-10-01")
        .with_end_date("2025-03-31")
        .with_current(true)
        .create(&config, &db)
        .await?;

    // Create courses
    let algo = CourseBuilder::new("algo", "Algorithms", 8)
        .in_semester(semester.id)
        .create(&db)
        .await?;

    let db_course = CourseBuilder::new("databases", "Database Systems", 6)
        .in_semester(semester.id)
        .create(&db)
        .await?;

    let networks = CourseBuilder::new("networks", "Computer Networks", 6)
        .in_semester(semester.id)
        .create(&db)
        .await?;

    println!("   ✓ Created degree, semester, and 3 courses\n");

    // Map courses to degree areas
    map_course_to_area(&db, algo.id, degree.id, degree.areas[0].id, None).await?;
    map_course_to_area(&db, db_course.id, degree.id, degree.areas[0].id, None).await?;
    map_course_to_area(&db, networks.id, degree.id, degree.areas[1].id, None).await?;

    // ================================================================================
    // RECORDING GRADES WITH COMPONENTS
    // ================================================================================

    println!("3. Recording grades with components...\n");

    // Example 1: German grade with weighted components
    println!("   Example 1: Algorithms (German grading with components)");
    let grade1 = GradeBuilder::new(algo.id, 0.0) // Will calculate from components
        .with_scheme(GradingScheme::German)
        .with_component("Midterm Exam", 0.4, Some(85.0), Some(100.0))
        .with_component("Final Exam", 0.6, Some(90.0), Some(100.0))
        .with_exam_date("2025-02-15")
        .as_final(true)
        .calculate_from_components() // Auto-calculate grade from components
        .record(&db)
        .await?;

    println!(
        "     Grade: {:.2} ({})",
        grade1.grade, grade1.grading_scheme
    );
    println!("     Passed: {}", grade1.passed);
    println!("     Components:");
    for comp in &grade1.components {
        println!(
            "       - {}: weight {:.1}%, earned {:.1}/{:.1}",
            comp.component_name,
            comp.weight * 100.0,
            comp.points_earned.unwrap_or(0.0),
            comp.points_total.unwrap_or(0.0)
        );
    }
    println!();

    // Example 2: Direct German grade without components
    println!("   Example 2: Database Systems (direct German grade)");
    let grade2 = GradeBuilder::new(db_course.id, 1.7)
        .with_scheme(GradingScheme::German)
        .with_exam_date("2025-02-20")
        .as_final(true)
        .record(&db)
        .await?;

    println!(
        "     Grade: {:.1} ({})",
        grade2.grade, grade2.grading_scheme
    );
    println!("     Passed: {}", grade2.passed);
    println!();

    // Example 3: US GPA with graded components
    println!("   Example 3: Computer Networks (US GPA with graded components)");
    let grade3 = GradeBuilder::new(networks.id, 0.0)
        .with_scheme(GradingScheme::US)
        .with_graded_component("Homework", 0.2, 3.7)
        .with_graded_component("Midterm", 0.3, 3.3)
        .with_graded_component("Final", 0.4, 3.5)
        .with_graded_component("Project", 0.1, 4.0)
        .with_bonus("Extra Credit", 0.2)
        .calculate_from_components()
        .as_final(true)
        .record(&db)
        .await?;

    println!(
        "     Grade: {:.2} ({})",
        grade3.grade, grade3.grading_scheme
    );
    println!("     Passed: {}", grade3.passed);
    println!("     Components:");
    for comp in &grade3.components {
        if comp.is_bonus {
            println!(
                "       - {} (BONUS): +{:.1} points",
                comp.component_name,
                comp.bonus_points.unwrap_or(0.0)
            );
        } else {
            println!(
                "       - {}: weight {:.1}%, grade {:.2}",
                comp.component_name,
                comp.weight * 100.0,
                comp.grade.unwrap_or(0.0)
            );
        }
    }
    println!();

    // ================================================================================
    // GRADE CONVERSION
    // ================================================================================

    println!("4. Grade conversion between schemes...\n");

    let german_grade = 1.7;
    println!("   Original: {:.1} (German)", german_grade);

    let ects_grade = german_to_ects(german_grade);
    println!("   → ECTS: {:?}", ects_grade);

    let us_gpa = convert_grade(german_grade, GradingScheme::German, GradingScheme::US);
    println!("   → US GPA: {:.2}", us_gpa.unwrap_or(0.0));

    let percentage = convert_grade(
        german_grade,
        GradingScheme::German,
        GradingScheme::Percentage,
    );
    println!("   → Percentage: {:.1}%", percentage.unwrap_or(0.0));

    println!();

    // Example: Converting US GPA to German
    let us_grade = 3.5;
    println!("   Original: {:.1} (US GPA)", us_grade);
    let german_converted = convert_grade(us_grade, GradingScheme::US, GradingScheme::German);
    println!("   → German: {:.2}", german_converted.unwrap_or(0.0));
    println!();

    // ================================================================================
    // QUERYING GRADES
    // ================================================================================

    println!("5. Querying grades...\n");

    // Get grade by ID
    let queried = get_grade_by_id(&db, grade1.id).await?;
    println!(
        "   Queried grade for {}: {:.2} ({})",
        "Algorithms", queried.grade, queried.grading_scheme
    );

    // Get final grade for a course
    let final_grade = get_final_grade(&db, algo.id).await?;
    if let Some(fg) = final_grade {
        println!("   Final grade for Algorithms: {:.2}", fg.grade);
    }
    println!();

    // ================================================================================
    // GPA CALCULATIONS
    // ================================================================================

    println!("6. GPA calculations...\n");

    // Overall GPA (only GPA-counting courses) - DEFAULT BEHAVIOR
    println!("   Overall GPA (German scheme, GPA-counting courses only):");
    let overall_gpa = calculate_overall_gpa(&db, GradingScheme::German, false).await?;
    println!("     GPA: {:.2}", overall_gpa.gpa);
    println!("     Courses: {}", overall_gpa.total_courses);
    println!("     Total ECTS: {}", overall_gpa.total_ects);
    println!();

    // Overall GPA including non-GPA courses
    println!("   Overall GPA (German scheme, INCLUDING non-GPA courses):");
    let overall_gpa_all = calculate_overall_gpa(&db, GradingScheme::German, true).await?;
    println!("     GPA: {:.2}", overall_gpa_all.gpa);
    println!("     Courses: {}", overall_gpa_all.total_courses);
    println!("     Total ECTS: {}", overall_gpa_all.total_ects);
    println!();

    // Semester GPA
    println!("   Semester GPA ({}, GPA-counting only): ", semester.code);
    let semester_gpa =
        calculate_semester_gpa(&db, semester.id, GradingScheme::German, false).await?;
    println!("     GPA: {:.2}", semester_gpa.gpa);
    println!("     Courses: {}", semester_gpa.total_courses);
    println!("     ECTS: {}", semester_gpa.total_ects);
    println!();

    println!("   Semester GPA ({}, including all): ", semester.code);
    let semester_gpa_all =
        calculate_semester_gpa(&db, semester.id, GradingScheme::German, true).await?;
    println!("     GPA: {:.2}", semester_gpa_all.gpa);
    println!("     Courses: {}", semester_gpa_all.total_courses);
    println!("     ECTS: {}", semester_gpa_all.total_ects);
    println!();

    // Degree GPA (automatically only counts GPA-counting areas)
    println!("   Degree GPA (Computer Science, GPA-counting areas only):");
    let degree_gpa = calculate_degree_gpa(&db, degree.id, GradingScheme::German).await?;
    println!("     GPA: {:.2}", degree_gpa.gpa);
    println!("     Courses counted: {}", degree_gpa.total_courses);
    println!("     ECTS counted: {}", degree_gpa.total_ects);
    println!();

    // US GPA equivalent
    println!("   Overall GPA (US scheme, GPA-counting only):");
    let us_overall = calculate_overall_gpa(&db, GradingScheme::US, false).await?;
    println!("     GPA: {:.2} / 4.0", us_overall.gpa);
    println!();

    // ================================================================================
    // RECORDING MULTIPLE ATTEMPTS
    // ================================================================================

    println!("7. Recording exam retakes...\n");

    // Create another course
    let physics = CourseBuilder::new("physics", "Physics I", 6)
        .in_semester(semester.id)
        .create(&db)
        .await?;

    // First attempt - failed
    let attempt1 = GradeBuilder::new(physics.id, 4.7)
        .with_scheme(GradingScheme::German)
        .with_attempt(1)
        .with_exam_date("2025-02-10")
        .as_final(false) // Not final, will retake
        .record(&db)
        .await?;

    println!("   First attempt: {:.1} (failed)", attempt1.grade);

    // Second attempt - passed
    let attempt2 = GradeBuilder::new(physics.id, 3.3)
        .with_scheme(GradingScheme::German)
        .with_attempt(2)
        .with_exam_date("2025-04-05")
        .as_final(true) // This is the final grade
        .record(&db)
        .await?;

    println!("   Second attempt: {:.1} (passed)", attempt2.grade);
    println!();

    // ================================================================================
    // DIFFERENT GRADING SCHEMES
    // ================================================================================

    println!("8. Examples of different grading schemes...\n");

    let test_course = CourseBuilder::new("test", "Test Course", 3)
        .in_semester(semester.id)
        .create(&db)
        .await?;

    // ECTS grading (A-F)
    println!("   ECTS grading (A=1, B=2, C=3, D=4, E=5, F=6):");
    let ects_grade_val = GradeBuilder::new(test_course.id, 2.0) // B
        .with_scheme(GradingScheme::ECTS)
        .as_final(true)
        .record(&db)
        .await?;
    println!("     Grade: {:.0} (B - Very Good)", ects_grade_val.grade);
    println!();

    // Percentage grading
    println!("   Percentage grading:");
    let pct_grade = GradeBuilder::new(test_course.id, 85.0)
        .with_scheme(GradingScheme::Percentage)
        .as_final(false)
        .record(&db)
        .await?;
    println!("     Grade: {:.0}%", pct_grade.grade);
    println!();

    // Pass/Fail
    println!("   Pass/Fail grading:");
    let pf_grade = GradeBuilder::new(test_course.id, 1.0) // 1 = pass
        .with_scheme(GradingScheme::PassFail)
        .as_final(false)
        .record(&db)
        .await?;
    println!(
        "     Grade: {} (Pass)",
        if pf_grade.grade >= 1.0 {
            "Pass"
        } else {
            "Fail"
        }
    );
    println!();

    // ================================================================================
    // SUMMARY
    // ================================================================================

    println!("=== Summary ===");
    println!("Recorded {} grades across {} courses:", 7, 5);
    println!(
        "  - Algorithms: {:.2} (German) - with components",
        grade1.grade
    );
    println!("  - Database Systems: {:.1} (German)", grade2.grade);
    println!(
        "  - Computer Networks: {:.2} (US GPA) - with components",
        grade3.grade
    );
    println!("  - Physics I: {:.1} (German, attempt 2)", attempt2.grade);
    println!("  - Test Course: Multiple schemes demonstrated");
    println!();
    println!("Overall GPA:");
    println!("  - German: {:.2}", overall_gpa.gpa);
    println!("  - US: {:.2} / 4.0", us_overall.gpa);
    println!("  - Total ECTS: {}", overall_gpa.total_ects);
    println!();
    println!("✓ All grade operations completed successfully!");
    println!();
    println!("Key features demonstrated:");
    println!("  ✓ Multiple grading schemes (German, US, ECTS, Percentage, Pass/Fail)");
    println!("  ✓ Grade components with weights");
    println!("  ✓ Automatic grade calculation from components");
    println!("  ✓ Bonus points");
    println!("  ✓ Grade conversions between schemes");
    println!("  ✓ Exam retakes/multiple attempts");
    println!("  ✓ GPA calculation (overall, semester, degree)");
    println!("  ✓ ECTS-weighted averages");

    Ok(())
}
