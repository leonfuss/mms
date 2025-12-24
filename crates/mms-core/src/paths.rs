use crate::error::{MmsError, Result};
use std::fs;
use std::path::{Path, PathBuf};

// === UTILITY ========================

fn ensure_exists(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .map_err(|e| MmsError::Config(format!("Failed to create data directory: {}", e)))
}

// ====================================

// === CONFIG BASE DIRECTORIES ========

/// Config directory base path
fn config_dir_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| MmsError::Config("Could not determine config directory".to_string()))?
        .join("mms");
    ensure_exists(&config_dir)?;
    Ok(config_dir)
}

/// Data directory base path
fn data_dir_path() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .ok_or_else(|| MmsError::Config("Could not determine data directory".to_string()))?
        .join("mms");
    ensure_exists(&data_dir)?;
    Ok(data_dir)
}

// ====================================

// === FILE PATHS =====================

/// Returns path to the config.toml file. File may not exist.
pub fn config_path() -> Result<PathBuf> {
    config_dir_path().map(|it| it.join("config.toml"))
}

/// Returns path to the database type (mms.db). File may not exist.
pub fn database_path() -> Result<PathBuf> {
    data_dir_path().map(|it| it.join("mms.db"))
}

/// Get the directory path for a course
/// Precondition: `base_path` must be canonicalized.
pub fn course_directory(
    base_path: &Path,
    semester_type_initial: char,
    semester_number: i32,
    course_short_name: &str,
) -> PathBuf {
    assert!(base_path.is_absolute(), "Paths must be canonicalized!");
    base_path
        .join(format!("{}{:02}", semester_type_initial, semester_number))
        .join(course_short_name)
}

// ====================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Paths must be canonicalized!")]
    fn test_course_directory_precondtion() {
        let base = PathBuf::from("uni");
        let _ = course_directory(&base, 'm', 5, "ml");
    }

    #[test]
    fn test_course_directory() {
        let base = PathBuf::from("/tmp/uni");
        let path = course_directory(&base, 'm', 5, "ml");
        assert_eq!(path, PathBuf::from("/tmp/uni/m05/ml"));

        let path = course_directory(&base, 'b', 2, "algo");
        assert_eq!(path, PathBuf::from("/tmp/uni/b02/algo"));
    }
}
