use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{MmsError, Result};
use crate::paths;

// ==================================================================================
// Configuration (Single validated struct)
// ==================================================================================

/// Application configuration.
///
/// Invariants enforced by the type system:
/// - `university_base_path` must exist (validated on construction via `load()`)
/// - All other fields are optional and will be `None` if not provided
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub university_base_path: PathBuf,
    pub general: Option<GeneralConfig>,
    pub grading: Option<GradingConfig>,
    pub notes: Option<NotesConfig>,
    pub schedule: Option<ScheduleConfig>,
    pub sync: Option<SyncConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub student_name: Option<String>,
    pub student_id: Option<String>,
    pub default_editor: Option<String>,
    pub default_pdf_viewer: Option<String>,
    pub default_location: Option<String>,
    pub symlink_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingConfig {
    pub default_scheme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesConfig {
    pub auto_watch: bool,
    pub auto_open_pdf: bool,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub auto_switch: bool,
    pub switch_window_minutes: u64,
    pub notify: bool,
    pub check_interval_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub auto_fetch: bool,
    pub platforms: Vec<String>,
}

// ==================================================================================
// Implementation
// ==================================================================================

impl Config {
    /// Load and validate config from the default location.
    ///
    /// # Errors
    /// Returns error if:
    /// - The config file does not exist or cannot be read
    /// - The TOML cannot be parsed
    /// - The required `university_base_path` field is missing or invalid
    ///
    /// # Postcondition
    /// The returned `Config` is guaranteed to have a valid `university_base_path`.
    pub fn load() -> Result<Self> {
        let config_path = paths::config_path()?;
        Self::load_from_path(&config_path)
    }

    /// Load and validate config from a specific path.
    ///
    /// # Errors
    /// Returns error if:
    /// - The config file does not exist or cannot be read
    /// - The TOML cannot be parsed
    /// - The required `university_base_path` field is missing or invalid
    ///
    /// # Postcondition
    /// The returned `Config` is guaranteed to have a valid `university_base_path`.
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(MmsError::ConfigNotFound {
                path: path.clone(),
            });
        }

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content).map_err(|e| MmsError::ConfigParseError {
            path: path.clone(),
            source: e,
        })?;

        // Enforce the invariant: university_base_path must be present and valid
        config.validate()?;
        Ok(config)
    }

    /// Validates that the required `university_base_path` field is set and its parent exists.
    ///
    /// This is the single point of validation, enforcing the type-level
    /// invariant that a `Config` always has this required field with a valid parent directory.
    fn validate(&self) -> Result<()> {
        if self.university_base_path.as_os_str().is_empty() {
            return Err(MmsError::UniversityBasePathMissing);
        }

        // Validate that the parent directory exists
        if let Some(parent) = self.university_base_path.parent() {
            if !parent.exists() {
                return Err(MmsError::UniversityBasePathParentNotFound {
                    path: self.university_base_path.clone(),
                    parent: parent.to_path_buf(),
                });
            }
        }

        Ok(())
    }

    /// Check if a valid config exists at the default location.
    pub fn exists() -> bool {
        Self::load().is_ok()
    }

    /// Save config to the default location.
    pub fn save(&self) -> Result<()> {
        let config_path = paths::config_path()?;
        self.save_to_path(&config_path)
    }

    /// Save config to a specific path.
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| MmsError::ConfigSerializeError {
                path: path.clone(),
                source: e,
            })?;

        fs::write(path, content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_empty_path_fails_validation() {
        let config = Config {
            university_base_path: PathBuf::new(),
            general: None,
            grading: None,
            notes: None,
            schedule: None,
            sync: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("university_base_path")
        );
    }

    #[test]
    fn test_nonexistent_parent_fails_validation() {
        let config = Config {
            university_base_path: PathBuf::from("/nonexistent/parent/path"),
            general: None,
            grading: None,
            notes: None,
            schedule: None,
            sync: None,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("University base path parent directory does not exist")
        );
    }

    #[test]
    fn test_valid_path_passes_validation() {
        let config = Config {
            university_base_path: PathBuf::from("/tmp"),
            general: None,
            grading: None,
            notes: None,
            schedule: None,
            sync: None,
        };

        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_with_all_optional_fields() {
        let config = Config {
            university_base_path: PathBuf::from("/tmp"),
            general: Some(GeneralConfig {
                student_name: Some("Alice".to_string()),
                student_id: Some("123".to_string()),
                default_editor: Some("vim".to_string()),
                default_pdf_viewer: Some("skim".to_string()),
                default_location: Some("University".to_string()),
                symlink_path: PathBuf::from_str("~").unwrap(),
            }),
            grading: Some(GradingConfig {
                default_scheme: "german".to_string(),
            }),
            notes: Some(NotesConfig {
                auto_watch: true,
                auto_open_pdf: true,
                template: "default".to_string(),
            }),
            schedule: Some(ScheduleConfig {
                auto_switch: true,
                switch_window_minutes: 10,
                notify: true,
                check_interval_minutes: 2,
            }),
            sync: Some(SyncConfig {
                auto_fetch: false,
                platforms: vec![],
            }),
        };

        let result = config.validate();
        assert!(result.is_ok());
        assert_eq!(config.university_base_path, PathBuf::from("/tmp"));
    }
}
