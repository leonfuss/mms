use anyhow::Result;
use colored::Colorize;
use inquire::{Select, Text, validator::Validation};
use mms_core::config::settings::{Config, PartialConfig};
use mms_core::error::MissingConfigFields;
use std::path::PathBuf;

pub fn ensure_config() -> Result<Config> {
    // 1. Try to load partial config
    let partial_res = PartialConfig::load();

    let partial = match partial_res {
        Ok(p) => p,
        Err(e) => {
            println!("{}", format!("Error loading config: {}", e).red());
            println!("Starting fresh configuration...\n");
            // Create empty partial
            PartialConfig {
                general: None,
                grading: None,
                notes: None,
                schedule: None,
                sync: None,
            }
        }
    };

    // 2. Validate
    match partial.clone().validate() {
        Ok(config) => Ok(config),
        Err(missing_fields) => {
            println!("{}", "Configuration is missing required fields.".yellow());
            println!("Missing: {}", missing_fields);
            println!("Let's set them up now.\n");

            run_setup_wizard(partial, missing_fields)
        }
    }
}

fn non_empty_validator(input: &str) -> Result<Validation, inquire::CustomUserError> {
    if input.trim().is_empty() {
        Ok(Validation::Invalid("This field cannot be empty".into()))
    } else {
        Ok(Validation::Valid)
    }
}

fn run_setup_wizard(partial: PartialConfig, missing: MissingConfigFields) -> Result<Config> {
    // Extract existing values if any
    let general = partial.general.clone();

    let current_name = general.as_ref().and_then(|g| g.student_name.clone());
    let current_id = general.as_ref().and_then(|g| g.student_id.clone());
    let current_root = general.as_ref().and_then(|g| g.university_base_path.clone());
    let current_editor = general.as_ref().and_then(|g| g.default_editor.clone());
    let current_pdf = general.as_ref().and_then(|g| g.default_pdf_viewer.clone());
    let current_location = general.as_ref().and_then(|g| g.default_location.clone());

    // Prompt 1: Student Name (only if missing)
    let student_name = if missing.student_name {
        Text::new("Student Name:")
            .with_default(current_name.as_deref().unwrap_or(""))
            .with_help_message("Your name for reports and git commits")
            .with_validator(non_empty_validator)
            .prompt()?
    } else {
        current_name.unwrap()
    };

    // Prompt 2: Student ID (only if missing)
    let student_id = if missing.student_id {
        Text::new("Student ID:")
            .with_default(current_id.as_deref().unwrap_or(""))
            .with_help_message("Matriculation number")
            .with_validator(non_empty_validator)
            .prompt()?
    } else {
        current_id.unwrap()
    };

    // Prompt 3: Studies Root (only if missing)
    let studies_root = if missing.studies_root {
        let default_root = dirs::home_dir()
            .map(|h| h.join("Studies").to_string_lossy().to_string())
            .unwrap_or_else(|| "~/Studies".to_string());

        let current_root_str = current_root
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(default_root);

        let studies_root_str = Text::new("Studies Directory:")
            .with_default(&current_root_str)
            .with_help_message("Absolute path or ~/Path")
            .with_validator(non_empty_validator)
            .prompt()?;

        PathBuf::from(&studies_root_str)
    } else {
        current_root.unwrap()
    };

    // Prompt 4: Editor (only if missing)
    let default_editor = if missing.default_editor {
        let editors = vec!["zed", "vim", "nano", "code", "emacs", "Other"];
        let default_editor_idx = if let Some(curr) = &current_editor {
            editors.iter().position(|&e| e == curr).unwrap_or(0)
        } else {
            0 // zed
        };

        let editor_selection = Select::new("Default Editor:", editors.clone())
            .with_starting_cursor(default_editor_idx)
            .prompt()?;

        if editor_selection == "Other" {
            Text::new("Enter editor command:")
                .with_default(current_editor.as_deref().unwrap_or("vim"))
                .with_validator(non_empty_validator)
                .prompt()?
        } else {
            editor_selection.to_string()
        }
    } else {
        current_editor.unwrap()
    };

    // Prompt 5: PDF Viewer (only if missing)
    let default_pdf_viewer = if missing.default_pdf_viewer {
        let viewers = vec!["skim", "preview", "zathura", "evince", "Other"];
        let default_viewer_idx = if let Some(curr) = &current_pdf {
            viewers.iter().position(|&v| v == curr).unwrap_or(0)
        } else {
            0 // skim
        };

        let viewer_selection = Select::new("Default PDF Viewer:", viewers.clone())
            .with_starting_cursor(default_viewer_idx)
            .prompt()?;

        if viewer_selection == "Other" {
            Text::new("Enter PDF viewer command:")
                .with_default(current_pdf.as_deref().unwrap_or("open"))
                .with_validator(non_empty_validator)
                .prompt()?
        } else {
            viewer_selection.to_string()
        }
    } else {
        current_pdf.unwrap()
    };

    // Prompt 6: Default Location (only if missing)
    let default_location = if missing.default_location {
        Text::new("Default Location:")
            .with_default(current_location.as_deref().unwrap_or("University"))
            .with_help_message("Default location for courses (e.g., 'TUM', 'University')")
            .with_validator(non_empty_validator)
            .prompt()?
    } else {
        current_location.unwrap()
    };

    // Construct valid Config with all required fields
    let config = Config::with_defaults(
        student_name,
        student_id,
        studies_root,
        default_editor,
        default_pdf_viewer,
        default_location,
    );

    // Save
    println!("\nSaving configuration...");
    config.save()?;
    println!("{}", "✓ Configuration saved successfully!".green());

    Ok(config)
}
