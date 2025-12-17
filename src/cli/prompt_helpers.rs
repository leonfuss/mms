/// Reusable interactive prompt helpers for CLI
///
/// This module provides consistent prompting patterns across the application

use crate::error::Result;
use dialoguer::{Input, Select, Confirm};

// ============================================================================
// Constants for common options
// ============================================================================

pub const DAYS_OF_WEEK: [&str; 7] = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];
pub const SCHEDULE_TYPES: [&str; 3] = ["lecture", "tutorium", "exercise"];

// ============================================================================
// Generic Prompts
// ============================================================================

/// Prompt for a required text input
pub fn prompt_text(label: &str) -> Result<String> {
    Ok(Input::<String>::new()
        .with_prompt(label)
        .interact_text()?)
}

/// Prompt for an optional text input
pub fn prompt_optional_text(label: &str, ask_first: bool) -> Result<Option<String>> {
    if ask_first {
        let add = Confirm::new()
            .with_prompt(format!("Add {}?", label.to_lowercase()))
            .default(false)
            .interact()?;

        if !add {
            return Ok(None);
        }
    }

    Ok(Some(Input::<String>::new()
        .with_prompt(label)
        .interact_text()?))
}

/// Prompt for selection from a list
pub fn prompt_select<T: std::fmt::Display>(label: &str, items: &[T]) -> Result<usize> {
    Ok(Select::new()
        .with_prompt(label)
        .items(items)
        .interact()?)
}

/// Prompt for selection from a list with default
pub fn prompt_select_with_default<T: std::fmt::Display>(label: &str, items: &[T], default: usize) -> Result<usize> {
    Ok(Select::new()
        .with_prompt(label)
        .items(items)
        .default(default)
        .interact()?)
}

/// Prompt for yes/no confirmation
pub fn prompt_confirm(label: &str, default: bool) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(label)
        .default(default)
        .interact()?)
}

// ============================================================================
// Time & Date Prompts
// ============================================================================

/// Prompt for time in HH:MM format
pub fn prompt_time(label: &str) -> Result<String> {
    Ok(Input::<String>::new()
        .with_prompt(format!("{} (HH:MM)", label))
        .interact_text()?)
}

/// Prompt for date in dd.mm.yyyy format
pub fn prompt_date(label: &str) -> Result<String> {
    Ok(Input::<String>::new()
        .with_prompt(format!("{} (dd.mm.yyyy)", label))
        .interact_text()?)
}

// ============================================================================
// Domain-Specific Prompts
// ============================================================================

/// Prompt for day of week selection
pub fn prompt_day_of_week() -> Result<String> {
    let selection = Select::new()
        .with_prompt("Day of week")
        .items(&DAYS_OF_WEEK)
        .interact()?;
    Ok(DAYS_OF_WEEK[selection].to_string())
}

/// Prompt for schedule type (lecture, tutorium, exercise)
pub fn prompt_schedule_type() -> Result<String> {
    let selection = Select::new()
        .with_prompt("Schedule type")
        .items(&SCHEDULE_TYPES)
        .default(0)
        .interact()?;
    Ok(SCHEDULE_TYPES[selection].to_string())
}

/// Prompt for optional room
pub fn prompt_room() -> Result<Option<String>> {
    prompt_optional_text("Room", true)
}

/// Prompt for optional location override
pub fn prompt_location_override() -> Result<Option<String>> {
    let add_location = Confirm::new()
        .with_prompt("Add location override?")
        .default(false)
        .interact()?;

    if add_location {
        Ok(Some(Input::<String>::new()
            .with_prompt("Location")
            .interact_text()?))
    } else {
        Ok(None)
    }
}

/// Prompt for optional description
pub fn prompt_description() -> Result<Option<String>> {
    prompt_optional_text("Description", true)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get value from Option or prompt interactively
pub fn get_or_prompt<F>(opt: Option<String>, prompt_fn: F) -> Result<String>
where
    F: FnOnce() -> Result<String>,
{
    if let Some(value) = opt {
        Ok(value)
    } else {
        prompt_fn()
    }
}

/// Get optional value or prompt interactively
pub fn get_or_prompt_optional<F>(opt: Option<String>, prompt_fn: F) -> Result<Option<String>>
where
    F: FnOnce() -> Result<Option<String>>,
{
    if opt.is_some() {
        Ok(opt)
    } else {
        prompt_fn()
    }
}
