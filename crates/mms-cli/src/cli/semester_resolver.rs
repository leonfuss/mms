use mms_core::config::Config;
use mms_core::db::connection;
use mms_core::db::models::semester::Semester;
use mms_core::db::queries;
use anyhow::Result;
use mms_core::error::MmsError;
use colored::Colorize;
use dialoguer::FuzzySelect;
use std::env;

/// Resolves a semester from various inputs: explicit ID, current directory, or interactive selection
pub struct SemesterResolver;

impl SemesterResolver {
    /// Resolve semester ID from optional input
    /// Priority:
    /// 1. If input is provided and is numeric, use as ID
    /// 2. If input is provided as shorthand (m01, b02, etc.), parse it
    /// 3. Check if current directory is within a semester folder
    /// 4. Fall back to current/active semester
    /// 5. If all fails, show fuzzy selection
    pub fn resolve(input: Option<String>) -> Result<(i64, Semester)> {
        let conn = connection::get()?;

        // 1. Try to parse input as ID or shorthand
        if let Some(ref input_str) = input {
            // Try parsing as numeric ID first
            if let Ok(id) = input_str.parse::<i64>() {
                let semester = queries::semester::get_by_id(&conn, id)?;
                return Ok((id, semester));
            }

            // Try parsing as shorthand (m01, b02, etc.)
            if let Some((id, semester)) = Self::parse_shorthand(input_str)? {
                println!("{} {}", "→ Using semester:".dimmed(), semester.to_string().cyan());
                return Ok((id, semester));
            }

            return Err(MmsError::Other(format!(
                "Could not find semester with ID or shorthand '{}'",
                input_str
            )).into());
        }

        // 2. Try to infer from current directory
        if let Some((id, semester)) = Self::infer_from_directory()? {
            println!("{} {} {}", "→ Inferred semester:".dimmed(), semester.to_string().cyan(), "(from current directory)".dimmed());
            return Ok((id, semester));
        }

        // 3. Try current semester
        if let Some(semester) = queries::semester::get_current(&conn)? {
            let id = semester.id.unwrap();
            println!("{} {} {}", "→ Using current semester:".dimmed(), semester.to_string().cyan(), "(no semester specified)".dimmed());
            return Ok((id, semester));
        }

        // 4. Show fuzzy selection
        Self::select_interactively()
    }

    /// Parse shorthand notation (m01, b02, etc.) to find matching semester
    /// Format: [m|b]<number> where m=master, b=bachelor
    fn parse_shorthand(shorthand: &str) -> Result<Option<(i64, Semester)>> {
        let conn = connection::get()?;
        let shorthand_lower = shorthand.to_lowercase();

        // Parse the shorthand format
        if shorthand_lower.len() < 2 {
            return Ok(None);
        }

        let type_char = shorthand_lower.chars().next().unwrap();
        let number_str = &shorthand_lower[1..];

        // Validate type character
        let semester_type = match type_char {
            'm' => "master",
            'b' => "bachelor",
            _ => return Ok(None),
        };

        // Parse the number
        let number = match number_str.parse::<i32>() {
            Ok(n) => n,
            Err(_) => return Ok(None),
        };

        // Find matching semester
        let semesters = queries::semester::list(&conn)?;
        for semester in semesters {
            let matches_type = match semester.type_ {
                mms_core::db::models::semester::SemesterType::Bachelor => semester_type == "bachelor",
                mms_core::db::models::semester::SemesterType::Master => semester_type == "master",
            };

            if matches_type && semester.number == number {
                return Ok(Some((semester.id.unwrap(), semester)));
            }
        }

        Ok(None)
    }

    /// Infer semester from current working directory
    /// Checks if we're inside a semester folder
    fn infer_from_directory() -> Result<Option<(i64, Semester)>> {
        let config = Config::load()?;
        let conn = connection::get()?;

        // Get current directory
        let current_dir = env::current_dir()?;
        let base_path = &config.general.university_base_path;

        // Check if we're within the university base path
        if !current_dir.starts_with(base_path) {
            return Ok(None);
        }

        // Get all semesters and check if current path matches any
        let semesters = queries::semester::list(&conn)?;

        for semester in semesters {
            let semester_path = base_path.join(semester.folder_name());

            // Check if current directory is the semester folder or a subdirectory
            if current_dir.starts_with(&semester_path) {
                return Ok(Some((semester.id.unwrap(), semester)));
            }
        }

        Ok(None)
    }

    /// Show interactive fuzzy selection of semesters
    fn select_interactively() -> Result<(i64, Semester)> {
        let conn = connection::get()?;
        let semesters = queries::semester::list(&conn)?;

        if semesters.is_empty() {
            return Err(MmsError::Other(
                "No semesters found. Create one first with 'mms semester add'".to_string(),
            ).into());
        }

        // Build selection items
        let items: Vec<String> = semesters
            .iter()
            .map(|s| s.to_string())
            .collect();

        println!();
        let selection = FuzzySelect::new()
            .with_prompt("Select a semester")
            .items(&items)
            .interact()?;

        let semester = &semesters[selection];
        Ok((semester.id.unwrap(), semester.clone()))
    }
}
