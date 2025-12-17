use crate::cli::args::CourseAction;
use crate::cli::semester_resolver::SemesterResolver;
use crate::cli::prompt_helpers::*;
use mms_core::config::Config;
use mms_core::db::connection;
use mms_core::db::models::course::Course;
use mms_core::db::queries;
use anyhow::Result;
use mms_core::error::MmsError;
use colored::Colorize;
use dialoguer::{Input, Select, Confirm};

pub fn handle(action: CourseAction) -> Result<()> {
    match action {
        CourseAction::Add => handle_add_interactive(),
        CourseAction::List { semester } => handle_list(semester),
        CourseAction::Show { id } => handle_show(id),
        CourseAction::Edit { id } => handle_update(id),
        CourseAction::Open { id } => handle_open(id),
        CourseAction::Grade { id, grade } => handle_set_grade(id, grade),
        CourseAction::SetActive { id } => handle_set_active(id),
    }
}

fn handle_add_interactive() -> Result<()> {
    let conn = connection::get()?;
    let config = Config::load()?;

    println!("{}", "Add New Course".bold().underline());
    println!();

    // Get current semester or let user choose
    let current_semester = queries::semester::get_current(&conn)?;
    let all_semesters = queries::semester::list(&conn)?;

    let semester = if all_semesters.is_empty() {
        return Err(MmsError::Other("No semesters found. Create a semester first with 'mms semester add'".to_string()).into());
    } else if let Some(current) = current_semester {
        let use_current = Confirm::new()
            .with_prompt(format!("Add to current semester ({})?", current.to_string()))
            .default(true)
            .interact()?;

        if use_current {
            current
        } else {
            // Let user select semester
            let semester_names: Vec<String> = all_semesters.iter()
                .map(|s| s.to_string())
                .collect();
            let selection = Select::new()
                .with_prompt("Select semester")
                .items(&semester_names)
                .interact()?;
            all_semesters[selection].clone()
        }
    } else {
        // No current semester, let user select
        let semester_names: Vec<String> = all_semesters.iter()
            .map(|s| s.to_string())
            .collect();
        let selection = Select::new()
            .with_prompt("Select semester")
            .items(&semester_names)
            .interact()?;
        all_semesters[selection].clone()
    };

    let semester_id = semester.id.unwrap();

    // Get course details
    let name = prompt_text("Course name (e.g., Machine Learning)")?;
    let short_name = prompt_text("Short name for folder (e.g., ML)")?;

    // Get available categories from config
    let config_cats: Vec<String> = config.categories.keys().cloned().collect();
    let category = if !config_cats.is_empty() {
        let selection = prompt_select("Category", &config_cats)?;
        config_cats[selection].clone()
    } else {
        prompt_text("Category")?
    };

    let ects: i32 = prompt_text("ECTS credits")?.parse()
        .map_err(|_| MmsError::Parse("Invalid ECTS value".to_string()))?;
    let add_lecturer = prompt_confirm("Add lecturer?", false)?;

    let lecturer = prompt_optional_text("Lecturer name", add_lecturer)?;

    let add_platform = prompt_confirm("Add learning platform URL?", false)?;
    let platform_url = prompt_optional_text("Platform URL (e.g., https://moodle.example.com/course/123)", add_platform)?;

    let mut course = Course::new(semester_id, name.clone(), short_name.clone(), category.clone(), ects);

    if let Some(lect) = lecturer {
        course = course.with_lecturer(lect);
    }

    if let Some(url) = platform_url {
        course = course.with_learning_platform(url);
    }

    println!();
    println!("{}", "Summary:".bold());
    println!("  Name:      {}", name);
    println!("  Short:     {}", short_name);
    println!("  Semester:  {}", semester.to_string());
    println!("  Category:  {}", category);
    println!("  ECTS:      {}", ects);
    println!();

    let confirm = Confirm::new()
        .with_prompt("Create this course?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("{}", "Cancelled.".yellow());
        return Ok(());
    }

    let id = queries::course::insert(&conn, &course)?;

    // Create course directory
    let semester_path = config.general.university_base_path.join(semester.folder_name());
    let course_path = semester_path.join(course.folder_name());
    std::fs::create_dir_all(&course_path)?;

    println!();
    println!("{}", "✓ Course created successfully!".green());
    println!("  ID:   {}", id);
    println!("  Path: {}", course_path.display().to_string().dimmed());

    Ok(())
}

fn handle_list(semester_input: Option<String>) -> Result<()> {
    let conn = connection::get()?;

    let courses = if semester_input.is_some() {
        let (semester_id, _) = SemesterResolver::resolve(semester_input)?;
        queries::course::list_by_semester(&conn, semester_id)?
    } else {
        queries::course::list(&conn)?
    };

    if courses.is_empty() {
        println!("{}", "No courses found.".yellow());
        println!("Use 'mms course add' to create one.");
        return Ok(());
    }

    // Get active course ID
    let active = queries::active::get(&conn)?;
    let active_course_id = active.course_id;

    println!("{}", "Courses:".bold().underline());
    println!();

    let mut last_semester_id: Option<i64> = None;

    for course in courses {
        // Print semester header if changed
        if Some(course.semester_id) != last_semester_id {
            if last_semester_id.is_some() {
                println!();
            }
            let semester = queries::semester::get_by_id(&conn, course.semester_id)?;
            println!("{}", format!("{}:", semester.to_string()).bold());
            last_semester_id = Some(course.semester_id);
        }

        let grade_str = if let Some(grade) = course.grade {
            format!(" - Grade: {:.1}", grade).green().to_string()
        } else {
            "".to_string()
        };

        let active_marker = if Some(course.id.unwrap()) == active_course_id {
            " ★".green()
        } else {
            "  ".normal()
        };

        println!(
            "{} [{}] {} ({}) - {} ECTS{}",
            active_marker,
            course.id.unwrap(),
            course.name.bold(),
            course.short_name.dimmed(),
            course.ects,
            grade_str
        );
    }

    Ok(())
}

fn handle_show(id: i64) -> Result<()> {
    let conn = connection::get()?;
    let course = queries::course::get_by_id(&conn, id)?;
    let semester = queries::semester::get_by_id(&conn, course.semester_id)?;

    println!("{}", "Course Details:".bold().underline());
    println!();
    println!("  ID:         {}", course.id.unwrap());
    println!("  Name:       {}", course.name.bold());
    println!("  Short Name: {}", course.short_name);
    println!("  Semester:   {}", semester.to_string());
    println!("  Category:   {}", course.category);
    println!("  ECTS:       {}", course.ects);

    if let Some(lecturer) = &course.lecturer {
        println!("  Lecturer:   {}", lecturer);
    }

    if let Some(location) = &course.location {
        println!("  Location:   {}", location);
    }

    if let Some(url) = &course.learning_platform_url {
        println!("  Platform:   {}", url.dimmed());
    }

    if let Some(grade) = course.grade {
        println!("  Grade:      {}", format!("{:.1}", grade).green().bold());
    }

    println!("  Counts towards average: {}", if course.counts_towards_average { "Yes" } else { "No" });

    Ok(())
}

fn handle_update(id: i64) -> Result<()> {
    let conn = connection::get()?;
    let mut course = queries::course::get_by_id(&conn, id)?;

    println!("{}", "Update Course".bold().underline());
    println!();
    println!("Leave blank to keep current value.");
    println!();

    // Update name
    let name_input: String = Input::new()
        .with_prompt(format!("Name [{}]", course.name))
        .allow_empty(true)
        .interact_text()?;
    if !name_input.is_empty() {
        course.name = name_input;
    }

    // Update lecturer
    let lecturer_prompt = format!(
        "Lecturer [{}] (type 'none' to remove)",
        course.lecturer.as_deref().unwrap_or("none")
    );
    let lecturer_input: String = Input::new()
        .with_prompt(lecturer_prompt)
        .allow_empty(true)
        .interact_text()?;
    if !lecturer_input.is_empty() {
        if lecturer_input.to_lowercase() == "none" {
            course.lecturer = None;
        } else {
            course.lecturer = Some(lecturer_input);
        }
    }

    // Update platform URL
    let platform_prompt = format!(
        "Platform URL [{}] (type 'none' to remove)",
        course.learning_platform_url.as_deref().unwrap_or("none")
    );
    let platform_input: String = Input::new()
        .with_prompt(platform_prompt)
        .allow_empty(true)
        .interact_text()?;
    if !platform_input.is_empty() {
        if platform_input.to_lowercase() == "none" {
            course.learning_platform_url = None;
        } else {
            course.learning_platform_url = Some(platform_input);
        }
    }

    // Update category
    let config = Config::load()?;
    let config_cats: Vec<String> = config.categories.keys().cloned().collect();

    let category_input: String = if !config_cats.is_empty() {
        let mut items = config_cats.clone();
        items.push(format!("Keep current ({})", course.category));
        items.push("Enter custom...".to_string());

        let selection = Select::new()
            .with_prompt("Category")
            .items(&items)
            .default(items.len() - 2)
            .interact()?;

        if selection == items.len() - 2 {
            // Keep current
            String::new()
        } else if selection == items.len() - 1 {
            // Custom input
            Input::new()
                .with_prompt(format!("Category [{}]", course.category))
                .allow_empty(true)
                .interact_text()?
        } else {
            config_cats[selection].clone()
        }
    } else {
        Input::new()
            .with_prompt(format!("Category [{}]", course.category))
            .allow_empty(true)
            .interact_text()?
    };

    if !category_input.is_empty() {
        course.category = category_input;
    }

    // Update ECTS
    let ects_input: String = Input::new()
        .with_prompt(format!("ECTS [{}]", course.ects))
        .allow_empty(true)
        .interact_text()?;
    if !ects_input.is_empty() {
        course.ects = ects_input.parse::<i32>()
            .map_err(|_| MmsError::Parse("Invalid ECTS value".to_string()))?;
    }

    // Update grade
    let grade_prompt = format!(
        "Grade [{}] (type 'none' to remove)",
        course.grade.map(|g| format!("{:.1}", g)).unwrap_or_else(|| "none".to_string())
    );
    let grade_input: String = Input::new()
        .with_prompt(grade_prompt)
        .allow_empty(true)
        .interact_text()?;
    if !grade_input.is_empty() {
        if grade_input.to_lowercase() == "none" {
            course.grade = None;
        } else {
            course.grade = Some(grade_input.parse::<f32>()
                .map_err(|_| MmsError::Parse("Invalid grade".to_string()))?);
        }
    }

    queries::course::update(&conn, &course)?;

    println!();
    println!("{}", "✓ Course updated successfully!".green());

    Ok(())
}

fn handle_set_grade(id: i64, grade: String) -> Result<()> {
    let conn = connection::get()?;
    let mut course = queries::course::get_by_id(&conn, id)?;

    if grade.to_lowercase() == "none" {
        course.grade = None;
        queries::course::update(&conn, &course)?;
        println!("{}", "✓ Grade removed!".green());
        println!("  {}", course.name.bold());
    } else {
        let grade_value = grade.parse::<f32>()
            .map_err(|_| MmsError::Parse(format!("Invalid grade: '{}'. Use a number or 'none'", grade)))?;
        course.grade = Some(grade_value);
        queries::course::update(&conn, &course)?;
        println!("{}", "✓ Grade updated!".green());
        println!("  {}: {}", course.name.bold(), format!("{:.1}", grade_value).green());
    }

    Ok(())
}

fn handle_open(id: i64) -> Result<()> {
    let conn = connection::get()?;
    let course = queries::course::get_by_id(&conn, id)?;

    if let Some(url) = &course.learning_platform_url {
        println!("Opening {}...", url);

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(url)
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(url)
                .spawn()?;
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(&["/C", "start", url])
                .spawn()?;
        }

        println!("{}", "✓ Opened in browser".green());
    } else {
        println!("{}", "No learning platform URL set for this course.".yellow());
        println!("Use 'mms course edit {}' to add one.", id);
    }

    Ok(())
}

fn handle_set_active(id: i64) -> Result<()> {
    let conn = connection::get()?;

    // Verify course exists
    let course = queries::course::get_by_id(&conn, id)?;
    let semester = queries::semester::get_by_id(&conn, course.semester_id)?;

    // Set as active (includes setting active semester)
    queries::active::set_active_course(&conn, id, course.semester_id)?;

    // Update both symlinks
    mms_core::symlink::update_semester_symlink(&semester.folder_name())?;
    mms_core::symlink::update_course_symlink(&semester.folder_name(), course.folder_name())?;

    println!("{}", "✓ Active course set!".green());
    println!("  {}", course.name.bold());
    println!("  {}", semester.to_string().dimmed());
    println!();
    println!("Symlinks updated:");
    println!("  ~/cs -> {}", semester.folder_name().dimmed());
    println!("  ~/cc -> {}/{}", semester.folder_name().dimmed(), course.folder_name().dimmed());

    Ok(())
}
