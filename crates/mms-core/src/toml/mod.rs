pub mod semester;
pub mod course;

pub use semester::{SemesterToml, SemesterType};
pub use course::CourseToml;

use crate::error::{MmsError, Result};
use std::path::Path;

/// Read and parse a TOML file
pub(crate) fn read_toml_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| MmsError::Other(format!("Failed to read TOML file {}: {}", path.display(), e)))?;

    toml::from_str(&content)
        .map_err(|e| MmsError::Config(format!("Failed to parse TOML file {}: {}", path.display(), e)))
}

/// Write a struct to a TOML file
pub(crate) fn write_toml_file<T: serde::Serialize>(path: &Path, data: &T) -> Result<()> {
    let content = toml::to_string_pretty(data)
        .map_err(|e| MmsError::Config(format!("Failed to serialize to TOML: {}", e)))?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| MmsError::Other(format!("Failed to create directory {}: {}", parent.display(), e)))?;
    }

    std::fs::write(path, content)
        .map_err(|e| MmsError::Other(format!("Failed to write TOML file {}: {}", path.display(), e)))
}
