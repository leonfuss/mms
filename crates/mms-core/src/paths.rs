use std::path::PathBuf;
use std::fs;
use crate::error::{MmsError, Result};

/// Get the default config path: ~/.config/mms/config.toml
pub fn default_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| MmsError::Config("Could not determine config directory".to_string()))?;

    Ok(config_dir.join("mms").join("config.toml"))
}

/// Get the database path: ~/.local/share/mms/mms.db
pub fn database_path() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| MmsError::Config("Could not determine data directory".to_string()))?;

    let db_dir = data_dir.join("mms");

    // Ensure directory exists
    fs::create_dir_all(&db_dir)
        .map_err(|e| MmsError::Config(format!("Failed to create data directory: {}", e)))?;

    Ok(db_dir.join("mms.db"))
}

/// Expand tilde in paths
pub fn expand_path(path: &PathBuf) -> PathBuf {
    if let Some(path_str) = path.to_str() {
        if path_str.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(&path_str[2..]);
            }
        }
    }
    path.clone()
}

/// Get the directory path for a course
pub fn get_course_directory(
    base_path: &PathBuf,
    semester_type_initial: char,
    semester_number: i32,
    course_short_name: &str,
) -> PathBuf {
    let expanded_base = expand_path(base_path);
    expanded_base
        .join(format!("{}{:02}", semester_type_initial, semester_number))
        .join(course_short_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path() {
        let path = PathBuf::from("~/test");
        let expanded = expand_path(&path);
        
        // Should not contain tilde if home dir is resolved
        if dirs::home_dir().is_some() {
            assert!(!expanded.to_str().unwrap().starts_with("~"));
        }
    }
    
    #[test]
    fn test_get_course_directory() {
        let base = PathBuf::from("/tmp/uni");
        let path = get_course_directory(&base, 'm', 5, "ml");
        assert_eq!(path, PathBuf::from("/tmp/uni/m05/ml"));
    }
}
