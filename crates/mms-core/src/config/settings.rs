use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{MissingConfigFields, MmsError, Result};
use crate::paths;

// ==================================================================================
// Validated Configuration (Used by the application)
// ==================================================================================

#[derive(Debug, Clone, Serialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub grading: GradingConfig,
    pub notes: NotesConfig,
    pub schedule: ScheduleConfig,
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneralConfig {
    #[serde(rename = "studies_root", alias = "university_base_path")]
    pub university_base_path: PathBuf,

    pub student_name: String,
    pub student_id: String,
    pub default_editor: String,
    pub default_pdf_viewer: String,

    pub default_location: String,
    pub symlink_path: PathBuf, // base path for symlinks. eg. /Users/leon/
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
// Partial Configuration (Used for parsing TOML)
// ==================================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct PartialConfig {
    pub general: Option<PartialGeneralConfig>,
    pub grading: Option<GradingConfig>, // These have safe defaults, so we can use them directly or Option
    pub notes: Option<NotesConfig>,
    pub schedule: Option<ScheduleConfig>,
    pub sync: Option<SyncConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartialGeneralConfig {
    #[serde(rename = "studies_root", alias = "university_base_path")]
    pub university_base_path: Option<PathBuf>,

    pub student_name: Option<String>,
    pub student_id: Option<String>,
    pub default_editor: Option<String>,
    pub default_pdf_viewer: Option<String>,

    pub default_location: Option<String>,
    pub symlink_path: Option<PathBuf>,
}

// ==================================================================================
// Defaults & Constants
// ==================================================================================

fn default_symlink_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join("cc")
    } else {
        PathBuf::from("/tmp/cc")
    }
}

// ==================================================================================
// Implementation
// ==================================================================================

impl PartialConfig {
    /// Load partial config from default path
    pub fn load() -> Result<Self> {
        let config_path = paths::config_path()?;
        Self::load_from_path(&config_path)
    }

    /// Load partial config from specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            // Return empty partial config (all Nones)
            return Ok(PartialConfig {
                general: None,
                grading: None,
                notes: None,
                schedule: None,
                sync: None,
            });
        }

        let content = fs::read_to_string(path)
            .map_err(|e| MmsError::Config(format!("Failed to read config file: {}", e)))?;

        let config: PartialConfig = toml::from_str(&content)
            .map_err(|e| MmsError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Validate and transform into Config
    /// Returns Ok(Config) or Err(MissingConfigFields) with specific missing fields
    pub fn validate(self) -> std::result::Result<Config, MissingConfigFields> {
        let mut missing = MissingConfigFields::new();

        // Extract general config or mark all fields as missing
        let general = if let Some(g) = self.general {
            let university_base_path = if let Some(path) = g.university_base_path {
                path
            } else {
                missing.studies_root = true;
                PathBuf::new()
            };

            let student_name = if let Some(name) = g.student_name {
                name
            } else {
                missing.student_name = true;
                String::new()
            };

            let student_id = if let Some(id) = g.student_id {
                id
            } else {
                missing.student_id = true;
                String::new()
            };

            let default_editor = if let Some(editor) = g.default_editor {
                editor
            } else {
                missing.default_editor = true;
                String::new()
            };

            let default_pdf_viewer = if let Some(viewer) = g.default_pdf_viewer {
                viewer
            } else {
                missing.default_pdf_viewer = true;
                String::new()
            };

            let default_location = if let Some(location) = g.default_location {
                location
            } else {
                missing.default_location = true;
                String::new()
            };

            let symlink_path = g.symlink_path.unwrap_or_else(default_symlink_path);

            GeneralConfig {
                university_base_path,
                student_name,
                student_id,
                default_editor,
                default_pdf_viewer,
                default_location,
                symlink_path,
            }
        } else {
            // Entire general section is missing
            missing.student_name = true;
            missing.student_id = true;
            missing.studies_root = true;
            missing.default_editor = true;
            missing.default_pdf_viewer = true;
            missing.default_location = true;

            GeneralConfig {
                university_base_path: PathBuf::new(),
                student_name: String::new(),
                student_id: String::new(),
                default_editor: String::new(),
                default_pdf_viewer: String::new(),
                default_location: String::new(),
                symlink_path: default_symlink_path(),
            }
        };

        if !missing.is_empty() {
            return Err(missing);
        }

        // Require all sections to be present
        let grading = self.grading.ok_or_else(|| {
            MissingConfigFields::new() // This is a different kind of error, should be handled separately
        })?;

        let notes = self.notes.ok_or_else(MissingConfigFields::new)?;

        let schedule = self.schedule.ok_or_else(MissingConfigFields::new)?;

        let sync = self.sync.ok_or_else(MissingConfigFields::new)?;

        Ok(Config {
            general,
            grading,
            notes,
            schedule,
            sync,
        })
    }
}

impl Config {
    /// Load config from the default location.
    /// This now enforces validation and returns error if configuration is incomplete.
    pub fn load() -> Result<Self> {
        let partial = PartialConfig::load()?;
        partial.validate().map_err(MmsError::ConfigIncomplete)
    }

    /// Check if a valid config exists
    pub fn exists() -> bool {
        Self::load().is_ok()
    }

    /// Save config to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = paths::config_path()?;
        self.save_to_path(&config_path)
    }

    /// Save config to a specific path
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                MmsError::Config(format!("Failed to create config directory: {}", e))
            })?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| MmsError::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content)
            .map_err(|e| MmsError::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }
}

impl Config {
    /// Create a new config with sensible defaults for initialization
    pub fn with_defaults(
        student_name: String,
        student_id: String,
        studies_root: PathBuf,
        default_editor: String,
        default_pdf_viewer: String,
        default_location: String,
    ) -> Self {
        Self {
            general: GeneralConfig {
                university_base_path: studies_root,
                student_name,
                student_id,
                default_editor,
                default_pdf_viewer,
                default_location,
                symlink_path: default_symlink_path(),
            },
            grading: GradingConfig {
                default_scheme: "german".to_string(),
            },
            notes: NotesConfig {
                auto_watch: true,
                auto_open_pdf: true,
                template: "default-lecture".to_string(),
            },
            schedule: ScheduleConfig {
                auto_switch: true,
                switch_window_minutes: 10,
                notify: true,
                check_interval_minutes: 2,
            },
            sync: SyncConfig {
                auto_fetch: false,
                platforms: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_load_empty() {
        let partial = PartialConfig {
            general: None,
            grading: None,
            notes: None,
            schedule: None,
            sync: None,
        };
        let res = partial.validate();
        assert!(res.is_err());
        let missing = res.unwrap_err();
        assert!(missing.student_name);
        assert!(missing.student_id);
        assert!(missing.studies_root);
    }

    #[test]
    fn test_partial_load_missing_fields() {
        let partial = PartialConfig {
            general: Some(PartialGeneralConfig {
                university_base_path: Some(PathBuf::from("/tmp")),
                student_name: None, // Missing
                student_id: Some("123".to_string()),
                default_editor: Some("vim".to_string()),
                default_pdf_viewer: Some("skim".to_string()),
                default_location: Some("University".to_string()),
                symlink_path: None,
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
        let res = partial.validate();
        assert!(res.is_err());
        let missing = res.unwrap_err();
        assert!(missing.student_name);
        assert!(!missing.student_id);
        assert!(!missing.studies_root);
    }

    #[test]
    fn test_valid_transformation() {
        let partial = PartialConfig {
            general: Some(PartialGeneralConfig {
                university_base_path: Some(PathBuf::from("/tmp")),
                student_name: Some("Alice".to_string()),
                student_id: Some("123".to_string()),
                default_editor: Some("vim".to_string()),
                default_pdf_viewer: Some("skim".to_string()),
                default_location: Some("University".to_string()),
                symlink_path: None,
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
        let res = partial.validate();
        assert!(res.is_ok());
        let config = res.unwrap();
        assert_eq!(config.general.student_name, "Alice");
        assert_eq!(config.general.default_editor, "vim");
    }
}
