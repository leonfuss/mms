use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::{MmsError, Result};
use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub service: ServiceConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub categories: HashMap<String, CategoryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub university_base_path: PathBuf,
    pub default_location: String,
    pub symlink_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub schedule_check_interval_minutes: u64,
    pub auto_commit_on_lecture_end: bool,
    pub auto_clear_todos_on_next_lecture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub author_name: String,
    pub author_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryConfig {
    pub required_ects: i32,
    pub counts_towards_average: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        Self {
            university_base_path: home.join("Documents/02_university"),
            default_location: "Uni Tübingen".to_string(),
            symlink_path: home.join("cc"),
        }
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            schedule_check_interval_minutes: 2,
            auto_commit_on_lecture_end: false,
            auto_clear_todos_on_next_lecture: true,
        }
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            author_name: "Student".to_string(),
            author_email: "student@example.com".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            service: ServiceConfig::default(),
            git: GitConfig::default(),
            categories: HashMap::new(),
        }
    }
}

impl Config {
    /// Load config from the default location: ~/.config/mms/config.toml
    pub fn load() -> Result<Self> {
        let config_path = paths::default_config_path()?;
        Self::load_from_path(&config_path)
    }

    /// Load config from a specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| MmsError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| MmsError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Save config to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = paths::default_config_path()?;
        self.save_to_path(&config_path)
    }

    /// Save config to a specific path
    pub fn save_to_path(&self, path: &PathBuf) -> Result<()> {
        // Ensure parent directory exists
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

    /// Initialize a new config file with default values and example categories
    pub fn init_default_config() -> Result<Self> {
        let mut config = Self::default();

        config.categories.insert(
            "ML_FOUND".to_string(),
            CategoryConfig {
                required_ects: 24,
                counts_towards_average: true,
            },
        );

        config.categories.insert(
            "ML_DIV".to_string(),
            CategoryConfig {
                required_ects: 36,
                counts_towards_average: true,
            },
        );

        config.categories.insert(
            "ML_CS".to_string(),
            CategoryConfig {
                required_ects: 18,
                counts_towards_average: true,
            },
        );

        config.categories.insert(
            "ML_EXP".to_string(),
            CategoryConfig {
                required_ects: 12,
                counts_towards_average: false,
            },
        );

        config.categories.insert(
            "ML_THESIS".to_string(),
            CategoryConfig {
                required_ects: 30,
                counts_towards_average: true,
            },
        );

        Ok(config)
    }

    /// Check if a config file exists at the default location
    pub fn exists() -> bool {
        if let Ok(path) = paths::default_config_path() {
            path.exists()
        } else {
            false
        }
    }

    /// Helper to get expanded university base path
    pub fn university_base_path(&self) -> PathBuf {
        paths::expand_path(&self.general.university_base_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.service.schedule_check_interval_minutes, 2);
        assert_eq!(config.general.default_location, "Uni Tübingen");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::init_default_config().unwrap();
        let toml_str = toml::to_string(&config).unwrap();

        // Should be able to deserialize it back
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.categories.len(), 5);
    }
}