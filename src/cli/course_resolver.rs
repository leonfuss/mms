use crate::config::Config;
use crate::db::connection;
use crate::db::models::course::Course;
use crate::db::queries;
use crate::error::{MmsError, Result};
use colored::Colorize;
use dialoguer::FuzzySelect;
use std::env;
use std::path::PathBuf;

/// Resolves a course from various inputs: explicit ID, shortname, current directory, or interactive selection
pub struct CourseResolver;

impl CourseResolver {
    /// Resolve course ID from optional input (ID or shortname)
    /// Priority:
    /// 1. If input is provided and is numeric, use as ID
    /// 2. If input is provided and is string, search by shortname
    /// 3. Check if current directory is within a course folder
    /// 4. Fall back to active course
    /// 5. If all fails, show fuzzy selection
    pub fn resolve(input: Option<String>) -> Result<(i64, Course)> {
        let conn = connection::get()?;

        // 1. Try to parse input as ID or shortname
        if let Some(ref input_str) = input {
            // Try parsing as ID first
            if let Ok(id) = input_str.parse::<i64>() {
                let course = queries::course::get_by_id(&conn, id)?;
                return Ok((id, course));
            }

            // Try matching by shortname
            if let Some((id, course)) = Self::find_by_shortname(input_str)? {
                println!("{} {}", "→ Using course:".dimmed(), course.name.cyan());
                return Ok((id, course));
            }

            return Err(MmsError::Other(format!(
                "Could not find course with ID or shortname '{}'",
                input_str
            )));
        }

        // 2. Try to infer from current directory
        if let Some((id, course)) = Self::infer_from_directory()? {
            println!("{} {} {}", "→ Inferred course:".dimmed(), course.name.cyan(), format!("(from current directory)").dimmed());
            return Ok((id, course));
        }

        // 3. Try active course
        let active = queries::active::get(&conn)?;
        if let Some(course_id) = active.course_id {
            let course = queries::course::get_by_id(&conn, course_id)?;
            println!("{} {} {}", "→ Using active course:".dimmed(), course.name.cyan(), "(no course specified)".dimmed());
            return Ok((course_id, course));
        }

        // 4. Show fuzzy selection
        Self::select_interactively()
    }

    /// Find course by shortname (case-insensitive)
    fn find_by_shortname(shortname: &str) -> Result<Option<(i64, Course)>> {
        let conn = connection::get()?;
        let courses = queries::course::list(&conn)?;

        for course in courses {
            if course.short_name.eq_ignore_ascii_case(shortname) {
                return Ok(Some((course.id.unwrap(), course)));
            }
        }

        Ok(None)
    }

    /// Infer course from current working directory
    /// Checks if we're inside a course folder by comparing paths
    fn infer_from_directory() -> Result<Option<(i64, Course)>> {
        let config = Config::load()?;
        let conn = connection::get()?;

        // Get current directory
        let current_dir = env::current_dir()?;
        let base_path = &config.general.university_base_path;

        // Check if we're within the university base path
        if !current_dir.starts_with(base_path) {
            return Ok(None);
        }

        // Get all courses and check if current path matches any
        let courses = queries::course::list(&conn)?;

        for course in courses {
            let semester = queries::semester::get_by_id(&conn, course.semester_id)?;
            let course_path = base_path
                .join(semester.folder_name())
                .join(course.folder_name());

            // Check if current directory is the course folder or a subdirectory
            if current_dir.starts_with(&course_path) {
                return Ok(Some((course.id.unwrap(), course)));
            }
        }

        Ok(None)
    }

    /// Show interactive fuzzy selection of courses
    fn select_interactively() -> Result<(i64, Course)> {
        let conn = connection::get()?;
        let courses = queries::course::list(&conn)?;

        if courses.is_empty() {
            return Err(MmsError::Other(
                "No courses found. Create one first with 'mms course add'".to_string(),
            ));
        }

        // Build selection items with semester info
        let mut items = Vec::new();
        let mut last_semester_id: Option<i64> = None;

        for course in &courses {
            if Some(course.semester_id) != last_semester_id {
                let semester = queries::semester::get_by_id(&conn, course.semester_id)?;
                items.push(format!("─── {} ───", semester.to_string()));
                last_semester_id = Some(course.semester_id);
            }
            items.push(format!(
                "  {} ({}) - {} ECTS",
                course.name, course.short_name, course.ects
            ));
        }

        println!();
        let selection = FuzzySelect::new()
            .with_prompt("Select a course")
            .items(&items)
            .interact()?;

        // Map back to actual course (skip separator lines)
        let mut course_index = 0;
        let mut item_index = 0;

        for (i, item) in items.iter().enumerate() {
            if i == selection {
                break;
            }
            if !item.starts_with("───") {
                course_index += 1;
            }
            item_index += 1;
        }

        // Adjust for separators
        let mut actual_index = 0;
        let mut seen_courses = 0;

        for (i, item) in items.iter().enumerate() {
            if !item.starts_with("───") {
                if seen_courses == course_index {
                    actual_index = i;
                    break;
                }
                seen_courses += 1;
            }
        }

        // Find the course at the selected index
        let mut current_course_idx = 0;
        for (i, item) in items.iter().enumerate() {
            if !item.starts_with("───") {
                if i == selection {
                    let course = &courses[current_course_idx];
                    return Ok((course.id.unwrap(), course.clone()));
                }
                current_course_idx += 1;
            }
        }

        Err(MmsError::Other("Invalid selection".to_string()))
    }

    /// Resolve course path for display purposes
    pub fn get_course_path(course_id: i64) -> Result<PathBuf> {
        let config = Config::load()?;
        let conn = connection::get()?;
        let course = queries::course::get_by_id(&conn, course_id)?;
        let semester = queries::semester::get_by_id(&conn, course.semester_id)?;

        Ok(config
            .general
            .university_base_path
            .join(semester.folder_name())
            .join(course.folder_name()))
    }
}
