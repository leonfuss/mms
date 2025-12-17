use crate::cli::args::SemesterAction;
use crate::cli::prompt_helpers::*;
use crate::db::models::semester::{Semester, SemesterType};
use crate::db::connection;
use crate::db::queries;
use crate::error::{MmsError, Result};
use colored::Colorize;

pub fn handle(action: SemesterAction) -> Result<()> {
    match action {
        SemesterAction::Add { type_, number, location } => {
            handle_add(type_, number, location)
        }
        SemesterAction::List => handle_list(),
        SemesterAction::SetCurrent { id } => handle_set_current(id),
    }
}

fn handle_add(type_str: String, number: i32, location: Option<String>) -> Result<()> {
    let semester_type = SemesterType::from_str(&type_str)
        .ok_or_else(|| MmsError::InvalidSemesterType(type_str.clone()))?;

    let conn = connection::get()?;

    // Load config to get default location and base path
    let config = crate::config::Config::load()?;
    let final_location = location.unwrap_or(config.general.default_location.clone());

    let semester = Semester {
        id: None,
        type_: semester_type,
        number,
        is_current: false,
        default_location: final_location,
        created_at: None,
    };

    let id = queries::semester::insert(&conn, &semester)?;

    // Create semester directory using folder_name (e.g., "m_01")
    let semester_path = config.general.university_base_path.join(semester.folder_name());
    std::fs::create_dir_all(&semester_path)?;

    println!("{}", "✓ Semester created successfully!".green());
    println!("  ID:       {}", id);
    println!("  Name:     {}", semester.to_string().bold());
    println!("  Location: {}", semester.default_location);
    println!("  Path:     {}", semester_path.display().to_string().dimmed());

    Ok(())
}

fn handle_list() -> Result<()> {
    let conn = connection::get()?;
    let mut semesters = queries::semester::list(&conn)?;

    if semesters.is_empty() {
        println!("{}", "No semesters found.".yellow());
        println!("Use 'mms semester add' to create one.");
        return Ok(());
    }

    // Separate and sort by type and number
    let mut master_semesters: Vec<_> = semesters
        .iter()
        .filter(|s| matches!(s.type_, SemesterType::Master))
        .collect();
    master_semesters.sort_by_key(|s| s.number);

    let mut bachelor_semesters: Vec<_> = semesters
        .iter()
        .filter(|s| matches!(s.type_, SemesterType::Bachelor))
        .collect();
    bachelor_semesters.sort_by_key(|s| s.number);

    // Helper function to format shorthand
    let format_shorthand = |semester: &Semester| -> String {
        let prefix = match semester.type_ {
            SemesterType::Master => "m",
            SemesterType::Bachelor => "b",
        };
        format!("{}{:02}", prefix, semester.number)
    };

    println!("{}", "Semesters:".bold().underline());
    println!();

    // Display Bachelor semesters first
    if !bachelor_semesters.is_empty() {
        for semester in bachelor_semesters {
            let current_marker = if semester.is_current { " ★".green() } else { "  ".normal() };
            let shorthand = format_shorthand(semester);
            println!(
                "{} {} ({}) - {}",
                current_marker,
                semester.to_string().bold(),
                shorthand.dimmed(),
                semester.default_location
            );
        }
        println!();
    }

    // Display Master semesters
    if !master_semesters.is_empty() {
        for semester in master_semesters {
            let current_marker = if semester.is_current { " ★".green() } else { "  ".normal() };
            let shorthand = format_shorthand(semester);
            println!(
                "{} {} ({}) - {}",
                current_marker,
                semester.to_string().bold(),
                shorthand.dimmed(),
                semester.default_location
            );
        }
    }

    Ok(())
}

fn handle_set_current(id: i64) -> Result<()> {
    let conn = connection::get()?;

    // Get semester details before setting it as current
    let semester = queries::semester::get_by_id(&conn, id)?;

    queries::semester::set_current(&conn, id)?;

    // Also update the active table (but clear the course)
    queries::active::set_active_semester(&conn, id)?;

    // Update the cs (current semester) symlink
    crate::symlink::update_semester_symlink(&semester.folder_name())?;

    // Remove the cc symlink since no course is active now
    let _ = crate::symlink::remove_course_symlink(); // Ignore error if doesn't exist

    println!("{}", "✓ Current semester updated!".green());
    println!("  {}", semester.to_string().bold());
    println!("  Location: {}", semester.default_location);
    println!("  Symlink:  ~/cs -> {}", semester.folder_name().dimmed());

    Ok(())
}

/// Interactive dialog for adding a semester
pub fn add_interactive() -> Result<()> {
    println!("{}", "Add New Semester".bold().underline());
    println!();

    let type_options = vec!["Bachelor", "Master"];
    let type_selection = prompt_select_with_default("Semester type", &type_options, 0)?;

    let semester_type = if type_selection == 0 {
        SemesterType::Bachelor
    } else {
        SemesterType::Master
    };

    let number: i32 = prompt_text("Semester number (e.g., 1, 2, 3...)")?.parse()
        .map_err(|_| MmsError::Parse("Invalid semester number".to_string()))?;

    // Load config to show default location
    let config = crate::config::Config::load()?;
    let default_loc = config.general.default_location.clone();

    let use_default_location = prompt_confirm(&format!("Use default location ({})?", default_loc), true)?;

    let location = if !use_default_location {
        Some(prompt_text("Location")?)
    } else {
        None
    };

    // Create preview semester to show display name
    let preview_semester = Semester {
        id: None,
        type_: semester_type,
        number,
        is_current: false,
        default_location: location.clone().unwrap_or(default_loc.clone()),
        created_at: None,
    };

    println!();
    println!("{}", "Summary:".bold());
    println!("  Name:     {}", preview_semester.to_string());
    println!("  Folder:   {}", preview_semester.folder_name());
    println!("  Location: {}", preview_semester.default_location);
    println!();

    let confirm = prompt_confirm("Create this semester?", true)?;

    if confirm {
        handle_add(semester_type.to_str().to_string(), number, location)?;
    } else {
        println!("{}", "Cancelled.".yellow());
    }

    Ok(())
}
