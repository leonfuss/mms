/// Example demonstrating the degree creation and management API
///
/// This example shows how to:
/// 1. Create a degree program with areas (e.g., Core CS, Electives)
/// 2. Create courses and map them to degree areas
/// 3. Track degree progress (ECTS earned per area)
/// 4. Update degree areas and mappings
/// 5. Find unmapped courses
///
/// Run with: cargo run --example degree_example

use mms_core::config::Config;
use mms_core::course::CourseBuilder;
use mms_core::db::connection_seaorm;
use mms_core::degree::{DegreeBuilder, DegreeType, get_degree_by_id, get_degree_progress, get_unmapped_courses, map_course_to_area};
use mms_core::semester::{SemesterBuilder, SemesterType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Degree Management API Example ===\n");

    // Load configuration and connect to database
    println!("1. Loading configuration and connecting to database...");
    let config = Config::load()?;
    let db = connection_seaorm::get_connection().await?;
    println!("   ✓ Connected!\n");

    // ================================================================================
    // CREATING A DEGREE PROGRAM
    // ================================================================================

    println!("2. Creating a Bachelor's degree in Computer Science...");
    let degree = DegreeBuilder::new(DegreeType::Bachelor, "Computer Science", "TUM")
        .with_total_ects(180)
        .with_start_date("2020-10-01")
        .with_expected_end_date("2023-09-30")
        .with_area("Core Computer Science", 60, true)     // Counts toward GPA
        .with_area("Mathematics", 30, true)               // Counts toward GPA
        .with_area("Electives", 30, false)                // Does NOT count toward GPA
        .with_area("Practical Courses", 30, true)         // Counts toward GPA
        .with_area("Bachelor's Thesis", 12, true)         // Counts toward GPA
        .with_area("General Studies", 18, false)          // Does NOT count toward GPA
        .create(&db)
        .await?;

    println!("   ✓ Created degree: {} - {}", degree.degree_type, degree.name);
    println!("     University: {}", degree.university);
    println!("     Total ECTS required: {}", degree.total_ects_required);
    println!("     Number of areas: {}", degree.areas.len());
    println!("\n   Areas defined:");
    for area in &degree.areas {
        let gpa_marker = if area.counts_towards_gpa { " [GPA]" } else { "" };
        println!("     - {} ({} ECTS){}",
            area.category_name,
            area.required_ects,
            gpa_marker
        );
    }
    println!();

    // ================================================================================
    // CREATING A SEMESTER AND COURSES
    // ================================================================================

    println!("3. Creating a semester and some courses...");
    let semester = SemesterBuilder::new(SemesterType::Bachelor, 3)
        .with_start_date("2021-10-01")
        .with_end_date("2022-03-31")
        .with_university("TUM")
        .with_current(true)
        .create(&config, &db)
        .await?;

    println!("   ✓ Created semester: {}\n", semester.code);

    // Create courses
    let course1 = CourseBuilder::new("algo", "Algorithms and Data Structures", 8)
        .in_semester(semester.id)
        .with_lecturer("Prof. Dr. Schmidt")
        .create(&config, &db)
        .await?;

    let course2 = CourseBuilder::new("linalg", "Linear Algebra", 8)
        .in_semester(semester.id)
        .with_lecturer("Prof. Dr. Wagner")
        .create(&config, &db)
        .await?;

    let course3 = CourseBuilder::new("software_eng", "Software Engineering Lab", 6)
        .in_semester(semester.id)
        .with_lecturer("Prof. Dr. Müller")
        .create(&config, &db)
        .await?;

    let course4 = CourseBuilder::new("music", "Music Appreciation", 3)
        .in_semester(semester.id)
        .with_lecturer("Prof. Dr. Bach")
        .create(&config, &db)
        .await?;

    println!("   ✓ Created 4 courses:");
    println!("     - {} ({} ECTS)", course1.short_name, course1.ects);
    println!("     - {} ({} ECTS)", course2.short_name, course2.ects);
    println!("     - {} ({} ECTS)", course3.short_name, course3.ects);
    println!("     - {} ({} ECTS)", course4.short_name, course4.ects);
    println!();

    // ================================================================================
    // MAPPING COURSES TO DEGREE AREAS
    // ================================================================================

    println!("4. Mapping courses to degree areas...");

    // Map algorithms to Core CS
    map_course_to_area(&db, course1.id, degree.areas[0].id, None).await?;
    println!("   ✓ Mapped '{}' → '{}'", course1.short_name, degree.areas[0].category_name);

    // Map linear algebra to Mathematics
    map_course_to_area(&db, course2.id, degree.areas[1].id, None).await?;
    println!("   ✓ Mapped '{}' → '{}'", course2.short_name, degree.areas[1].category_name);

    // Map software engineering to Practical Courses
    map_course_to_area(&db, course3.id, degree.areas[3].id, None).await?;
    println!("   ✓ Mapped '{}' → '{}'", course3.short_name, degree.areas[3].category_name);

    // Map music to General Studies (does not count toward GPA)
    map_course_to_area(&db, course4.id, degree.areas[5].id, None).await?;
    println!("   ✓ Mapped '{}' → '{}' (non-GPA)", course4.short_name, degree.areas[5].category_name);
    println!();

    // ================================================================================
    // CHECKING UNMAPPED COURSES
    // ================================================================================

    println!("5. Checking for unmapped courses...");
    let unmapped = get_unmapped_courses(&db, degree.id).await?;

    if unmapped.is_empty() {
        println!("   ✓ All courses are mapped to degree areas!");
    } else {
        println!("   Found {} unmapped course(s):", unmapped.len());
        for course in unmapped {
            println!("     - {} - {}", course.short_name, course.name);
        }
    }
    println!();

    // ================================================================================
    // VIEWING DEGREE PROGRESS
    // ================================================================================

    println!("6. Viewing degree progress...");
    let progress = get_degree_progress(&db, degree.id).await?;

    println!("   Degree: {}", progress.degree_name);
    println!("   Total ECTS: {}/{} ({:.1}%)",
        progress.total_ects_earned,
        progress.total_ects_required,
        (progress.total_ects_earned as f64 / progress.total_ects_required as f64) * 100.0
    );

    if let Some(gpa) = progress.overall_gpa {
        println!("   Overall GPA: {:.2}", gpa);
    } else {
        println!("   Overall GPA: Not available (no grades yet)");
    }

    println!("\n   Progress by area:");
    for area_progress in &progress.area_progress {
        let percentage = if area_progress.required_ects > 0 {
            (area_progress.earned_ects as f64 / area_progress.required_ects as f64) * 100.0
        } else {
            0.0
        };

        let gpa_marker = if area_progress.counts_towards_gpa { " [GPA]" } else { "" };

        println!("     - {}: {}/{} ECTS ({:.1}%){}",
            area_progress.category_name,
            area_progress.earned_ects,
            area_progress.required_ects,
            percentage,
            gpa_marker
        );

        if let Some(gpa) = area_progress.area_gpa {
            println!("       Area GPA: {:.2}", gpa);
        }
    }
    println!();

    // ================================================================================
    // QUERYING DEGREE DETAILS
    // ================================================================================

    println!("7. Querying degree details...");
    let queried_degree = get_degree_by_id(&db, degree.id).await?;

    println!("   Degree ID: {}", queried_degree.id);
    println!("   Type: {}", queried_degree.degree_type);
    println!("   Name: {}", queried_degree.name);
    println!("   University: {}", queried_degree.university);
    println!("   Active: {}", queried_degree.is_active);

    if let Some(start) = &queried_degree.start_date {
        println!("   Start date: {}", start);
    }
    if let Some(end) = &queried_degree.expected_end_date {
        println!("   Expected end: {}", end);
    }
    println!();

    // ================================================================================
    // EXAMPLE WITH ECTS OVERRIDE
    // ================================================================================

    println!("8. Example: Mapping with ECTS override...");
    println!("   (This allows a course to count for different ECTS in a specific area)");

    // Create another course
    let course5 = CourseBuilder::new("seminar", "Advanced Topics Seminar", 5)
        .in_semester(semester.id)
        .create(&config, &db)
        .await?;

    // Map it to electives, but only count 3 ECTS toward that area
    map_course_to_area(&db, course5.id, degree.areas[2].id, Some(3)).await?;
    println!("   ✓ Mapped '{}' (5 ECTS) → '{}' (counts as 3 ECTS)",
        course5.short_name,
        degree.areas[2].category_name
    );
    println!();

    // ================================================================================
    // CREATING ADDITIONAL DEGREE TYPES
    // ================================================================================

    println!("9. Creating other degree types...");

    // Master's degree
    let master = DegreeBuilder::new(DegreeType::Master, "Data Science", "TUM")
        .with_total_ects(120)  // Masters typically need 120 ECTS
        .with_area("Core Data Science", 40, true)
        .with_area("Specialization", 30, true)
        .with_area("Electives", 20, false)
        .with_area("Master's Thesis", 30, true)
        .create(&db)
        .await?;

    println!("   ✓ Created Master's degree: {}", master.name);

    // PhD (doesn't typically have ECTS)
    let phd = DegreeBuilder::new(DegreeType::PhD, "Computer Science", "TUM")
        .with_total_ects(0)  // PhDs don't have ECTS requirements
        .create(&db)
        .await?;

    println!("   ✓ Created PhD: {}", phd.name);
    println!();

    // ================================================================================
    // SUMMARY
    // ================================================================================

    println!("=== Summary ===");
    println!("Created 3 degree programs:");
    println!("  - Bachelor's in Computer Science (180 ECTS, 6 areas)");
    println!("  - Master's in Data Science (120 ECTS, 4 areas)");
    println!("  - PhD in Computer Science (no ECTS requirement)");
    println!();
    println!("Created 5 courses and mapped 4 to the Bachelor's degree:");
    println!("  - {} → Core Computer Science", course1.short_name);
    println!("  - {} → Mathematics", course2.short_name);
    println!("  - {} → Practical Courses", course3.short_name);
    println!("  - {} → General Studies (non-GPA)", course4.short_name);
    println!("  - {} → Electives (3/5 ECTS counted)", course5.short_name);
    println!();
    println!("✓ All degree operations completed successfully!");

    Ok(())
}
