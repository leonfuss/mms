/// Example demonstrating the course creation and management API
///
/// This example shows how to:
/// 1. Create a semester first (courses need to belong to a semester)
/// 2. Create courses (folder + TOML + database)
/// 3. Update courses
/// 4. Query courses
/// 5. Delete courses
///
/// Run with: cargo run --example course_example
use mms_core::config::Config;
use mms_core::course::{CourseBuilder, get_course_by_short_name, list_courses, update_course};
use mms_core::db::connection_seaorm;
use mms_core::semester::{SemesterBuilder, SemesterType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Course API Example ===\n");

    // Load configuration and connect to database
    println!("1. Loading configuration and connecting to database...");
    let config = Config::load()?;
    let db = connection_seaorm::get_connection().await?;
    println!("   ✓ Connected!\n");

    // ================================================================================
    // CREATING A SEMESTER (Required for courses)
    // ================================================================================

    println!("2. Creating a semester first (courses need a semester)...");
    let semester = SemesterBuilder::new(SemesterType::Bachelor, 5)
        .with_start_date("2025-04-01")
        .with_end_date("2025-09-30")
        .with_university("TUM")
        .with_current(true)
        .create(&config, &db)
        .await?;

    println!(
        "   ✓ Created semester: {} (ID: {})",
        semester.code, semester.id
    );
    println!("   ✓ Directory: {:?}\n", semester.directory_path);

    // ================================================================================
    // CREATING COURSES
    // ================================================================================

    println!("3. Creating courses using CourseBuilder...");

    // Course 1: Full featured course
    let course1 = CourseBuilder::new("cs101", "Introduction to Algorithms", 8)
        .in_semester(semester.id)
        .with_lecturer("Prof. Dr. Schmidt")
        .with_lecturer_email("schmidt@tum.de")
        .with_tutor("Anna Müller")
        .with_tutor_email("anna@tum.de")
        .with_learning_platform_url("https://moodle.tum.de/course/12345")
        .with_university("TUM")
        .with_git_repo("https://github.com/tum/cs101")
        .create(&config, &db)
        .await?;

    println!("   ✓ Created course: {}", course1.short_name);
    println!("     Name: {}", course1.name);
    println!("     ECTS: {}", course1.ects);
    println!("     Directory: {:?}", course1.directory_path);
    println!("     TOML file: {:?}", course1.toml_path);
    println!("     Lecturer: {:?}", course1.lecturer);
    println!("     Git repo: {:?}", course1.git_remote_url);
    println!();

    // Course 2: Minimal course
    let course2 = CourseBuilder::new("math201", "Linear Algebra", 6)
        .in_semester(semester.id)
        .with_lecturer("Prof. Wagner")
        .create(&config, &db)
        .await?;

    println!(
        "   ✓ Created course: {} - {}",
        course2.short_name, course2.name
    );
    println!("     ECTS: {}", course2.ects);
    println!();

    // Course 3: External course (imported from elsewhere)
    let course3 = CourseBuilder::new("phys101", "Physics I", 5)
        .in_semester(semester.id)
        .as_external(Some("/old/studies/physics".to_string()))
        .create(&config, &db)
        .await?;

    println!("   ✓ Created external course: {}", course3.short_name);
    println!("     Is external: {}", course3.is_external);
    println!("     Original path: {:?}", course3.original_path);
    println!();

    // ================================================================================
    // QUERYING COURSES
    // ================================================================================

    println!("4. Querying courses...");

    // Query by short name
    let queried = get_course_by_short_name(&db, semester.id, "cs101").await?;
    println!(
        "   Found course by short name: {} - {}",
        queried.short_name, queried.name
    );
    println!("   Tutor: {:?}", queried.tutor);
    println!();

    // List all courses in semester
    let all_courses = list_courses(&db, Some(semester.id), false, false).await?;
    println!(
        "   Total courses in semester {}: {}",
        semester.code,
        all_courses.len()
    );
    for course in &all_courses {
        println!(
            "     - {} ({} ECTS) - {}",
            course.short_name, course.ects, course.name
        );
    }
    println!();

    // ================================================================================
    // UPDATING A COURSE
    // ================================================================================

    println!("5. Updating a course...");

    let updated = update_course(
        &db,
        course1.id,
        Some("Advanced Algorithms".to_string()), // New name
        Some(10),                                // Updated ECTS
        None,                                    // Keep lecturer
        None,                                    // Keep lecturer email
        Some("Thomas Weber".to_string()),        // New tutor
        Some("thomas@tum.de".to_string()),       // New tutor email
        Some("https://moodle.tum.de/course/67890".to_string()), // Updated URL
        None,                                    // Keep university
        None,                                    // Keep location
    )
    .await?;

    println!("   ✓ Updated course: {}", updated.short_name);
    println!("     New name: {}", updated.name);
    println!("     New ECTS: {}", updated.ects);
    println!("     New tutor: {:?}", updated.tutor);
    println!("     New tutor email: {:?}", updated.tutor_email);
    println!(
        "     Updated platform URL: {:?}",
        updated.learning_platform_url
    );
    println!();

    // ================================================================================
    // ALTERNATIVE: USING create_course FUNCTION DIRECTLY
    // ================================================================================

    println!("6. Alternative: Using create_course function directly...");
    use mms_core::course::create_course;

    let course4 = create_course(
        &db,
        semester.id,
        "db101".to_string(),
        "Database Systems".to_string(),
        6,
        Some("Prof. Müller".to_string()),
        Some("mueller@tum.de".to_string()),
        None, // No tutor
        None,
        None, // No learning platform
        Some("TUM".to_string()),
        None,
        false, // Not external
        None,
        false, // No git repo
        None,
    )
    .await?;

    println!(
        "   ✓ Created course: {} - {}",
        course4.short_name, course4.name
    );
    println!();

    // ================================================================================
    // SUMMARY
    // ================================================================================

    println!("=== Summary ===");
    println!("Created 1 semester and 4 courses:");
    println!(
        "\nSemester: {} ({:?})",
        semester.code, semester.directory_path
    );

    let final_list = list_courses(&db, Some(semester.id), false, false).await?;
    println!("\nCourses:");
    for course in &final_list {
        let external_marker = if course.is_external {
            " [EXTERNAL]"
        } else {
            ""
        };
        let git_marker = if course.has_git_repo { " [GIT]" } else { "" };
        println!(
            "  {} - {} ({} ECTS){}{}",
            course.short_name, course.name, course.ects, external_marker, git_marker
        );
        println!("    Directory: {:?}", course.directory_path);
        if let Some(toml) = &course.toml_path {
            println!("    TOML: {:?}", toml);
        }
    }

    println!("\n✓ All operations completed successfully!");
    println!("\nCheck the course directories:");
    println!("  Semester: {:?}", semester.directory_path);
    for course in &final_list {
        println!("  - {:?}", course.directory_path);
    }
    println!("\nEach non-external course directory should contain a .course.toml file.");

    Ok(())
}
